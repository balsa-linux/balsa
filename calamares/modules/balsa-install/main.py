import subprocess
import libcalamares

# Pool name from cfggen's disko template.
ZFS_POOL = "zroot"

def run():
    gs = libcalamares.globalstorage
    root = gs.value("rootMountPoint")
    config = gs.value("balsaConfigPath")
    if not config:
        return ("No generated config", "balsa-install ran before balsa-config-export wrote a flake.")

    # cfggen/src/templates/flake.nix.j2 names the system after the hostname.
    hostname = gs.value("hostname")

    # nixos-install writes flake.lock into the tree it just hashed (NAR hash mismatch), so lock first.
    lock = subprocess.run(["nix", "flake", "lock", "path:" + config],
                          capture_output=True, text=True)
    if lock.returncode != 0:
        return ("flake lock failed", lock.stderr)

    cmd = [
        "pkexec",
        "nixos-install",
        "--no-root-passwd",
        "--root",
        root,
        "--flake",
        "{}#{}".format(config, hostname),
        # Nix rejects build dirs under world-writable parents, like the chroot store's /tmp.
        "--option",
        "build-dir",
        "/nix/var/nix/builds",
    ]

    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    output = ""
    for line in proc.stdout:
        output += line
        libcalamares.utils.debug("nixos-install: {}".format(line.strip()))

    if proc.wait() != 0:
        return ("nixos-install failed", output)

    # Teardown lives here, not in the stock umount job, which fails on an already-unmounted target.
    unmount = subprocess.run(["umount", "-R", root], capture_output=True, text=True)
    if unmount.returncode != 0:
        return ("Unmounting the target failed", unmount.stderr)

    # Unexported, the pool still belongs to the live ISO's hostid and the installed system won't import it.
    if gs.value("balsaFilesystem") == "zfs":
        export = subprocess.run(["zpool", "export", ZFS_POOL], capture_output=True, text=True)
        if export.returncode != 0:
            return ("Exporting the ZFS pool failed", export.stderr)
    return None
