import glob
import os
import subprocess
import libcalamares

CONFIGGEN = "balsa-configgen"
DISKO = "disko"
OUT_DIR = "/tmp/balsa-disko"
ROOT_MOUNT = "/mnt"
# cfggen's disko template reads the passphrase from here.
PASSWORD_FILE = "/tmp/disko-password"
SUPPORTED_FS = ("btrfs", "ext4", "xfs", "zfs")


def _parent_disk(device):
    disk = device.rstrip("0123456789")
    # nvme0n1p2 and mmcblk0p1 separate the partition number with a "p"; sda2 does not.
    if disk.endswith("p") and len(disk) > 1 and disk[-2].isdigit():
        disk = disk[:-1]
    return disk


def _swap_size_gib(device):
    # The partitions map has no sizes, but the partition job already made the layout, so sysfs has them.
    path = "/sys/class/block/{}/size".format(os.path.basename(device))
    with open(path) as f:
        sectors = int(f.read())
    return max(1, round(sectors * 512 / 2 ** 30))


def _release(disk):
    # A previous attempt in the same session leaves the pool imported, and disko then reuses it.
    subprocess.run(["umount", "-R", ROOT_MOUNT], capture_output=True, text=True)
    subprocess.run(["swapoff", "-a"], capture_output=True, text=True)
    subprocess.run(["zpool", "export", "-a"], capture_output=True, text=True)
    # ZFS labels sit at both ends of a member, so wipefs leaves them for the next layout to trip on.
    for device in [disk] + sorted(set(glob.glob(disk + "*")) - {disk}):
        subprocess.run(["zpool", "labelclear", "-f", device], capture_output=True, text=True)


def run():
    gs = libcalamares.globalstorage
    partitions = gs.value("partitions") or []

    root = next((p for p in partitions if p.get("mountPoint") == "/"), None)
    if root is None:
        return ("Disk selection missing",
                "balsa-disko found no partition mounted at / in globalstorage.")

    fs = root.get("fs")
    if fs not in SUPPORTED_FS:
        return ("Unsupported root filesystem",
                "balsa-configgen has no disko layout for root filesystem {!r}.".format(fs))

    if not root.get("device"):
        return ("Disk selection missing",
                "The partition mounted at / has no device path in globalstorage.")

    disk = _parent_disk(root["device"])
    luks_name = ""
    if root.get("fsName") in ("luks", "luks2"):
        luks_name = root.get("luksMapperName") or "cryptroot"
        passphrase = root.get("luksPassphrase")
        if not passphrase:
            return ("Encryption passphrase missing",
                    "The root partition is {} but carries no passphrase.".format(root["fsName"]))
        with open(os.open(PASSWORD_FILE, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), "w") as f:
            f.write(passphrase)
    swap = next((p for p in partitions if p.get("fs") == "linuxswap"), None)
    swap_device = (swap or {}).get("device") or ""
    swap_size_gib = 0
    if swap is None:
        swap_mode = "none"
    elif swap_device.startswith("/dev/zram"):
        swap_mode = "zram"
    else:
        swap_mode = "partition"
        try:
            swap_size_gib = _swap_size_gib(swap_device)
        except (OSError, ValueError) as e:
            return ("Swap size unknown",
                    "Could not read the size of swap partition {}: {}".format(swap_device, e))

    esp = next((p for p in partitions
                if p.get("device") == gs.value("efiSystemPartition")), None)
    esp_mount = esp.get("mountPoint") if esp else None

    args = [CONFIGGEN, "disko", "--disk", disk, "--fs", fs, "--out", OUT_DIR,
            "--swap", swap_mode]
    if swap_mode == "partition":
        args += ["--swap-size-gib", str(swap_size_gib)]
    if luks_name:
        args += ["--luks-name", luks_name]
    if gs.value("firmwareType") != "efi":
        args.append("--legacy-bios")
    if esp_mount:
        args += ["--esp-mount", esp_mount]

    gen = subprocess.run(args, capture_output=True, text=True)
    if gen.returncode != 0:
        return ("disko config generation failed", gen.stderr)

    config_path = gen.stdout.strip()

    _release(disk)

    # --yes-wipe-all-disks skips a tty prompt no job can answer; Calamares already asked the user.
    apply = subprocess.run(
        [DISKO, "--mode", "destroy,format,mount",
         "--root-mountpoint", ROOT_MOUNT,
         "--yes-wipe-all-disks", config_path],
        capture_output=True, text=True,
    )
    if apply.returncode != 0:
        return ("disko apply failed", apply.stderr)

    # Nothing else sets this now that the stock mount module is out of the sequence.
    gs.insert("rootMountPoint", ROOT_MOUNT)

    # balsa-config-export reads these, not gs["partitions"], so disko's layout and the plan can't diverge.
    gs.insert("balsaDiskoConfigPath", config_path)
    gs.insert("balsaTargetDisk", disk)
    gs.insert("balsaFilesystem", fs)
    gs.insert("balsaSwapMode", swap_mode)
    gs.insert("balsaSwapSizeGib", swap_size_gib)
    gs.insert("balsaLuksName", luks_name)
    gs.insert("balsaEspMount", esp_mount or "/boot")
    return None
