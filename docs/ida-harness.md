# IDA harness

The harness is included here as a read-only utility. Keep its generated reports beside the private IDA database and do not commit them.

Its output is used to maintain the derived findings in this repository.

## Local invocation

    & 'C:\Program Files\IDA Professional 9.2\idat.exe' -A -S'D:\gta6-vr-port\tools\ida\vr_static_harness.py' 'C:\path\to\private\analysis.i64'

The harness is static-only. It enumerates imports, selected strings, cross-references, render candidates, and the checked-in camera, frustum, culling, presentation, pose, and bone-map target inventory. It does not patch or execute the binary.

The target inventory uses image-relative RVAs and includes xrefs for each entry. Generated reports therefore provide a ready-made review list for the first live trace while keeping the executable and generated report private.
