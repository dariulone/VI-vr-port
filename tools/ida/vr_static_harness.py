"""Static VR-oriented inventory for the Xbox Scarlett game executable.

Run inside IDA 9.x after auto-analysis:
  idat.exe -A -Svr_static_harness.py -Lida.log -Ovr_out:<path> <binary>

The script only reads the database and writes JSON and Markdown reports. It does
not patch the executable or execute the game.
"""

import json
import os
import re
from collections import defaultdict

import ida_bytes
import ida_funcs
import ida_idaapi
import ida_kernwin
import ida_nalt
import ida_name
import idaapi
import idautils
import idc


DEFAULT_OUT = os.path.join(os.getcwd(), "vr_static_report")


def output_dir():
    env_out = os.environ.get("VR_STATIC_HARNESS_OUT")
    if env_out:
        return env_out
    options = idaapi.get_plugin_options("vr_static_harness") or ""
    if options.startswith("out="):
        return options[4:]
    return DEFAULT_OUT


def safe_name(ea):
    name = ida_name.get_name(ea) or ""
    return name[:240]


def function_record(ea):
    f = ida_funcs.get_func(ea)
    if f is None:
        return {"ea": hex(ea), "rva": hex(ea - idaapi.get_imagebase()), "name": safe_name(ea)}
    return {
        "ea": hex(ea),
        "rva": hex(ea - idaapi.get_imagebase()),
        "name": safe_name(ea),
        "start": hex(f.start_ea),
        "end": hex(f.end_ea),
    }


def import_records():
    rows = []
    for module_index in range(ida_nalt.get_import_module_qty()):
        module = ida_nalt.get_import_module_name(module_index) or ""
        names = []

        def collect(ea, name, ordinal):
            text = name.decode("utf-8", "replace") if isinstance(name, bytes) else str(name or "")
            name_ea = ea
            callers = []
            for ref in idautils.XrefsTo(name_ea, 0):
                caller = ida_funcs.get_func(ref.frm)
                if caller:
                    callers.append(function_record(caller.start_ea))
            names.append({"name": text, "ordinal": ordinal, "ea": hex(name_ea), "callers": callers[:40]})
            return True

        ida_nalt.enum_import_names(module_index, collect)
        rows.append({"module": module, "imports": names})
    return rows


def strings_matching(patterns):
    compiled = [re.compile(pattern, re.I) for pattern in patterns]
    rows = []
    seen = set()
    for string in idautils.Strings():
        value = str(string)
        if not any(regex.search(value) for regex in compiled):
            continue
        ea = int(string.ea)
        if ea in seen:
            continue
        seen.add(ea)
        xrefs = []
        for ref in idautils.XrefsTo(ea, 0):
            caller = ida_funcs.get_func(ref.frm)
            xrefs.append(function_record(caller.start_ea if caller else ref.frm))
        rows.append({
            "ea": hex(ea),
            "rva": hex(ea - idaapi.get_imagebase()),
            "text": value[:500],
            "xrefs": xrefs[:60],
        })
    return rows


def imported_candidates(imports, patterns):
    compiled = [re.compile(pattern, re.I) for pattern in patterns]
    candidates = {}
    for row in imports:
        for item in row["imports"]:
            if not any(regex.search(row["module"] + "!" + item["name"]) for regex in compiled):
                continue
            for caller in item["callers"]:
                key = caller["ea"]
                current = candidates.setdefault(key, dict(caller, hits=[]))
                current["hits"].append(row["module"] + "!" + item["name"])
    return sorted(candidates.values(), key=lambda value: (value["name"], value["ea"]))


def report_markdown(report):
    lines = [
        "# Static VR-oriented report",
        "",
        f"Input {report['input_file']}",
        f"Image base {report['imagebase']}",
        "",
        "## Imported render and platform APIs",
        "",
    ]
    for item in report["render_candidates"]:
        lines.append(f"- `{item['ea']}` `{item['name']}` hits {', '.join(item['hits'])}")
    lines.extend(["", "## Relevant strings", ""])
    for item in report["strings"]:
        lines.append(f"- `{item['rva']}` `{item['text']}`")
        for caller in item["xrefs"][:8]:
            lines.append(f"  - xref `{caller['rva']}` `{caller['name']}`")
    lines.extend(["", "## Platform and VR assessment", ""])
    lines.append("- OpenXR/OpenVR/SteamVR references are reported only when present in the executable strings or imports.")
    lines.append("- D3D12/Xbox render calls are grouped by containing function for the next review pass.")
    lines.append("- Camera/projection strings are evidence for candidate search only and require pseudocode review.")
    return "\n".join(lines) + "\n"


def main():
    output = output_dir()
    os.makedirs(output, exist_ok=True)
    strings = strings_matching([
        r"openxr", r"openvr", r"steamvr", r"xr_[a-z0-9_]+", r"presentx", r"swapchain",
        r"camera", r"projection", r"view.?matrix", r"stereo", r"eye", r"head", r"gameinput",
        r"d3d12", r"xg_xs", r"xmem", r"xapu", r"chad",
    ])
    imports = import_records()
    render_candidates = imported_candidates(imports, [
        r"d3d12", r"present", r"swap", r"xg_xs", r"xmem", r"xapu", r"chad", r"gameinput",
    ])
    report = {
        "input_file": ida_nalt.get_input_file_path(),
        "imagebase": hex(idaapi.get_imagebase()),
        "function_count": sum(1 for _ in idautils.Functions()),
        "imports": imports,
        "render_candidates": render_candidates,
        "strings": strings,
    }
    json_path = os.path.join(output, "vr_static_report.json")
    md_path = os.path.join(output, "vr_static_report.md")
    with open(json_path, "w", encoding="utf-8") as handle:
        json.dump(report, handle, ensure_ascii=False, indent=2)
    with open(md_path, "w", encoding="utf-8") as handle:
        handle.write(report_markdown(report))
    print("VR_STATIC_HARNESS_JSON", json_path)
    print("VR_STATIC_HARNESS_MD", md_path)
    print("VR_STATIC_FUNCTIONS", report["function_count"])
    print("VR_STATIC_IMPORT_MODULES", len(imports))
    print("VR_STATIC_CANDIDATES", len(render_candidates))
    print("VR_STATIC_STRINGS", len(strings))


if __name__ == "__main__":
    main()
