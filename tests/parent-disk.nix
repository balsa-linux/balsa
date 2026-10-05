# balsa-disko derives the whole-disk path from a partition, and nvme/mmcblk
# naming is what broke it, so the check needs real nvme nodes in sysfs.
pkgs:

let
  partitioned = pkgs.runCommand "nvme-partitioned.raw" { nativeBuildInputs = [ pkgs.util-linux ]; } ''
    truncate -s 128M $out
    sfdisk $out <<EOF
    label: gpt
    ,32M,L
    ,,L
    EOF
  '';

  blank = pkgs.runCommand "nvme-blank.raw" { } "truncate -s 64M $out";

  check = pkgs.writeText "check-parent-disk.py" ''
    import importlib.util
    import shlex
    import subprocess
    import sys
    import types

    sys.modules["libcalamares"] = types.ModuleType("libcalamares")
    spec = importlib.util.spec_from_file_location("bd", sys.argv[1])
    bd = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(bd)

    out = subprocess.run(["lsblk", "-Pno", "NAME,PKNAME,TYPE"],
                         capture_output=True, text=True, check=True).stdout
    seen = {}
    for line in out.splitlines():
        if not line.strip():
            continue
        d = dict(kv.split("=", 1) for kv in shlex.split(line))
        name, pk = d["NAME"], d["PKNAME"]
        want = "/dev/" + (pk if pk else name)
        got = bd._parent_disk("/dev/" + name)
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
  '';
in
pkgs.testers.runNixOSTest {
  name = "balsa-parent-disk";

  nodes.machine = {
    # virtualisation.qemu.drives only emits virtio-blk-pci or scsi-hd.
    virtualisation.qemu.options = [
      "-drive file=${partitioned},if=none,id=nvmep,format=raw,snapshot=on"
      "-device nvme,drive=nvmep,serial=parted"
      "-drive file=${blank},if=none,id=nvmeb,format=raw,snapshot=on"
      "-device nvme,drive=nvmeb,serial=blank"
    ];
    environment.systemPackages = [ pkgs.util-linux ];
  };

  testScript = ''
    machine.wait_for_unit("multi-user.target")
    print(machine.succeed("lsblk -o NAME,TYPE,SIZE"))
    print(machine.succeed("${pkgs.python3.interpreter} ${check} ${../calamares/modules/balsa-disko/main.py}"))
  '';
}
