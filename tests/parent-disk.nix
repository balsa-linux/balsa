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
    print(machine.succeed("${pkgs.python3.interpreter} ${./parent-disk.py} ${../calamares/modules/balsa-disko/main.py}"))
  '';
}
