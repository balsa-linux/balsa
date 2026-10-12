import runpy
import shlex
import subprocess
import sys
import types

sys.modules["libcalamares"] = types.ModuleType("libcalamares")
bd = runpy.run_path(sys.argv[1])

out = subprocess.run(["lsblk", "-Pno", "NAME,PKNAME,TYPE"],
                     capture_output=True, text=True, check=True).stdout
seen = {}
for line in out.splitlines():
    if not line.strip():
        continue
    d = dict(kv.split("=", 1) for kv in shlex.split(line))
    name, pk = d["NAME"], d["PKNAME"]
    want = "/dev/" + (pk if pk else name)
    got = bd["_parent_disk"]("/dev/" + name)
    print("{:12s} {:6s} -> {:14s} {}".format(
        name, d["TYPE"], got, "ok" if got == want else "MISMATCH want " + want))
    seen[name] = (got, want)

failures = [n for n, (g, w) in seen.items() if g != w]
missing = [n for n in ["nvme0n1", "nvme0n1p1", "nvme0n1p2", "nvme1n1"] if n not in seen]
if missing:
    print("MISSING DEVICES:", missing)
if failures:
    print("FAILURES:", failures)
assert not missing and not failures
print("PASS: {} devices, nvme partition and whole-disk cases included".format(len(seen)))
