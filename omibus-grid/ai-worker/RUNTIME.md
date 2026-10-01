# Bundled runtime

CPython 3.14.7, official Windows x64 embeddable distribution.

- Source: https://www.python.org/ftp/python/3.14.7/python-3.14.7-embed-amd64.zip
- SHA-256, verified before extraction: `d297e5ff019966817ad8502465176139f2d3d840fa4ed84b13bed399a6ab1f15`
- Release and published checksum: https://www.python.org/downloads/release/python-3147/
- License: `runtime/LICENSE.txt`.

The worker uses only the standard library and runs with `-I` in its own process.
It adds its own source directory to the import path; the runtime's default `_pth`
is unchanged. No system Python, pip, package installation, shell command strings,
exchange keys, or order APIs are used. Keep `ai-worker` next to the executable.

For source checkouts, retrieve the same official ZIP, verify that hash, and
extract it into `ai-worker/runtime`. Runtime binaries are excluded from Git.
