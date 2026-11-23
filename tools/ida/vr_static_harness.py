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


STATIC_TARGETS = [
    ("device_init", 0x2478440),
    ("frame_begin", 0x41E9E0),
    ("worker", 0x2481E90),
    ("present_lifecycle", 0x2484180),
    ("present_core", 0x24826A0),
    ("camera_owner_tick", 0x2008AE0),
    ("camera_render_prepare", 0x3231B30),
    ("camera_packet_copy", 0x53159B0),
    ("camera_visibility_consumer", 0x31B0B0),
    ("camera_frustum_prepare", 0x4128B0),
    ("camera_frustum_shape_math", 0x4440C60),
    ("frustum_resource_dispatch", 0x4E29640),
    ("frustum_range_dispatch", 0x4E296B0),
    ("frustum_range_process", 0x2DAB0),
    ("instance_culling_dispatch", 0x25D8590),
    ("pose_source", 0x523B630),
    ("pose_copy", 0x523B680),
    ("pose_finalize", 0x523BD00),
    ("pose_aux_array_build", 0x523C690),
    ("pose_gate_update", 0x523C810),
    ("bone_hash_map", 0x523C6F0),
    ("vehicle_request_system", 0x5814350),
    ("vehicle_deform_init", 0xCBF1F0),
    ("vehicle_deform_layout_a", 0x5B02F40),
    ("vehicle_deform_layout_b", 0x5432060),
    ("explosion_damage_init", 0x2511AB0),
    ("gpu_particle_collision_init", 0x1AF3BA0),
    ("rain_collision_init", 0x1D87EF0),
    ("rain_collision_resource", 0x5D44230),
    ("rain_persistent_resource", 0x5C0C5F0),
    ("water_simulation_init", 0x1E490F0),
    ("water_system_state", 0x26205C0),
    ("water_state_builder", 0x5B83070),
    ("water_system_helper", 0x2620140),
    ("constraint_name_resolver", 0x26870F0),
    ("horse_collision_response", 0x1D8C9B0),
]


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


def static_target_records():
    imagebase = idaapi.get_imagebase()
    rows = []
    for role, rva in STATIC_TARGETS:
        ea = imagebase + rva
        refs = []
        for ref in idautils.XrefsTo(ea, 0):
            caller = ida_funcs.get_func(ref.frm)
            refs.append(function_record(caller.start_ea if caller else ref.frm))
        rows.append({
            "role": role,
            "ea": hex(ea),
            "rva": hex(rva),
            "function": function_record(ea),
            "xrefs": refs[:80],
        })
    return rows


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
    lines.extend(["", "## Static VR target inventory", ""])
    for item in report["static_targets"]:
        function = item["function"]
        lines.append(f"- `{item['role']}` RVA `{item['rva']}` `{function['name']}` xrefs {len(item['xrefs'])}")
        for caller in item["xrefs"][:6]:
            lines.append(f"  - xref `{caller['rva']}` `{caller['name']}`")
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
    static_targets = static_target_records()
    render_candidates = imported_candidates(imports, [
        r"d3d12", r"present", r"swap", r"xg_xs", r"xmem", r"xapu", r"chad", r"gameinput",
    ])
    report = {
        "input_file": ida_nalt.get_input_file_path(),
        "imagebase": hex(idaapi.get_imagebase()),
        "function_count": sum(1 for _ in idautils.Functions()),
        "imports": imports,
        "render_candidates": render_candidates,
        "static_targets": static_targets,
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
    print("VR_STATIC_TARGETS", len(static_targets))
    print("VR_STATIC_STRINGS", len(strings))


if __name__ == "__main__":
    main()
