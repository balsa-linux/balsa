import json
import os
import platform
import secrets
import subprocess
import libcalamares

CONFIGGEN = "balsa-configgen"
MKPASSWD = "mkpasswd"
DEST = "etc/nixos"  # inside the installed root
PLAN = "/tmp/balsa-plan.toml"
# Written by iso/default.nix from system.nixos.revision.
NIXPKGS_REV_FILE = "/etc/balsa/nixpkgs-rev"
BALSA_REV_FILE = "/etc/balsa/balsa-rev"

DESKTOPS = {"plasma6": "plasma"}  # every other chooser id already matches DesktopChoice

KERNELS = {
    "default": {"type": "default"},
    "lts": {"type": "lts"},
    "zen": {"type": "zen"},
    "xanmod": {"type": "xanmod"},
    "cachyos": {"type": "cachy-os", "variant": "latest"},
    "cachyos-lts": {"type": "cachy-os", "variant": "lts"},
    "cachyos-bore": {"type": "cachy-os", "variant": "bore"},
}

CPUS = {"intel": "intel", "amd": "amd", "unknown": "other"}

GPUS = {"intel": "intel", "amd": "amd", "nvidia": "nvidia-proprietary"}

ARCHES = {"x86_64": "x86_64-linux", "aarch64": "aarch64-linux"}


def _deobscure(s):
    # Calamares' obscure() (libcalamares/utils/String.cpp) is a self-inverse transform, not a hash.
    return "".join(c if ord(c) <= 0x21 else chr(0x1001F - ord(c)) for c in s)


def _fmt(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, int):
        return str(v)
    if isinstance(v, str):
        # TOML basic strings and JSON strings escape identically for plain text and hashes.
        return json.dumps(v)
    if isinstance(v, list):
        return "[" + ", ".join(_fmt(x) for x in v) + "]"
    raise TypeError("cannot serialise {!r} as TOML".format(v))


def _dump(table, prefix=""):
    lines = ["{} = {}".format(k, _fmt(v)) for k, v in table.items() if not isinstance(v, dict)]
    for k, v in table.items():
        if isinstance(v, dict):
            name = prefix + k
            lines += ["", "[" + name + "]"] + _dump(v, name + ".")
    return lines


def _disk(gs):
    # Reuse balsa-disko's resolved layout so the config describes the disk disko actually made.
    disk = gs.value("balsaTargetDisk")
    if not disk:
        return None, ("No disk layout",
                      "balsa-config-export ran before balsa-disko partitioned the target.")

    out = {
        "disk": disk,
        "filesystem": gs.value("balsaFilesystem"),
        "scheme": {"type": "guided"},
    }

    mode = gs.value("balsaSwapMode")
    if mode == "partition":
        out["swap"] = {"type": "partition", "size_gib": gs.value("balsaSwapSizeGib")}
    else:
        out["swap"] = {"type": mode}

    luks_name = gs.value("balsaLuksName")
    if luks_name:
        out["encryption"] = {"luks_name": luks_name}

    return out, None


def run():
    gs = libcalamares.globalstorage
    root = gs.value("rootMountPoint")
    if not root:
        return ("No root mount point", "balsa-config-export ran before the target was mounted.")

    disk, err = _disk(gs)
    if err:
        return err

    desktop_id = gs.value("balsaDesktop")
    desktop = DESKTOPS.get(desktop_id, desktop_id)
    kernel_id = gs.value("balsaKernel")
    if kernel_id not in KERNELS:
        return ("Unknown kernel choice", "No plan mapping for kernel id {!r}.".format(kernel_id))

    password = _deobscure(gs.value("password") or "")
    if not password:
        return ("No password set", "The users page left globalstorage['password'] empty.")

    hashed = subprocess.run(
        [MKPASSWD, "-m", "yescrypt", "-s"],
        input=password + "\n", capture_output=True, text=True,
    )
    if hashed.returncode != 0:
        # stderr can echo the password.
        return ("Password hashing failed", "mkpasswd exited {}.".format(hashed.returncode))
    user_password = hashed.stdout.strip()

    # No separate root password reaches globalstorage, so root can only reuse the user's hash.
    root_password = user_password if gs.value("reuseRootPassword") else ""

    locale_conf = gs.value("localeConf") or {}
    legacy_bios = gs.value("balsaFirmware") == "bios"

    # Name a missing value instead of a TypeError from the TOML writer.
    required = {
        "hostname": gs.value("hostname"),
        "username": gs.value("username"),
        "balsaDesktop": desktop,
        "balsaTuningDefault": gs.value("balsaTuningDefault"),
        "locationRegion": gs.value("locationRegion"),
        "locationZone": gs.value("locationZone"),
    }
    missing = sorted(k for k, v in required.items() if not v)
    if missing:
        return ("Incomplete install plan",
                "globalstorage has no value for: {}.".format(", ".join(missing)))

    network = {
        "hostname": gs.value("hostname"),
        "backend": "network-manager",
    }
    if disk["filesystem"] == "zfs":
        network["host_id"] = secrets.token_hex(4)

    accounts = {
        "root_enabled": bool(root_password),
        "primary": {
            "username": gs.value("username"),
            "full_name": gs.value("fullname") or "",
            "hashed_password": user_password,
        },
    }
    if root_password:
        accounts["root_hashed_password"] = root_password

    plan = {
        "desktop": desktop,
        "tuning_profile": gs.value("balsaTuningDefault"),
        "locale": {
            "locale": locale_conf.get("LANG", "en_US.UTF-8").split("/")[0],
            "keymap": gs.value("keyboardLayout") or "us",
            "timezone": "{}/{}".format(gs.value("locationRegion"), gs.value("locationZone")),
        },
        "disk": disk,
        "boot": {
            # systemd-boot cannot boot legacy BIOS, so GRUB takes over there.
            "loader": "grub" if legacy_bios else "systemd-boot",
            "legacy_bios": legacy_bios,
            "esp_mount": gs.value("balsaEspMount") or "/boot",
        },
        "hardware": {
            "arch": ARCHES.get(platform.machine(), "x86_64-linux"),
            "cpu": CPUS.get(gs.value("balsaCpuVendor"), "other"),
            "gpu": [GPUS[v] for v in (gs.value("balsaGpuVendors") or []) if v in GPUS],
            # The plan's firmware field is the blob policy, not UEFI vs BIOS.
            "firmware": "auto",
        },
        "network": network,
        "kernel": KERNELS[kernel_id],
        "accounts": accounts,
    }
    login = gs.value("balsaLoginManager")
    if login:
        plan["login_manager"] = login

    # Pin to the ISO's nixpkgs, so nixos-install reuses the live store, and to its Balsa commit.
    for key, path in (("nixpkgs_ref", NIXPKGS_REV_FILE), ("balsa_ref", BALSA_REV_FILE)):
        try:
            with open(path) as f:
                plan[key] = f.read().strip()
        except OSError as e:
            return ("ISO revision unknown", "Could not read {}: {}".format(path, e))
        if not plan[key]:
            return ("ISO revision unknown", "{} is empty.".format(path))

    with open(PLAN, "w") as f:
        f.write("\n".join(_dump(plan)) + "\n")

    out = os.path.join(root, DEST)
    os.makedirs(out, exist_ok=True)

    result = subprocess.run(
        [CONFIGGEN, "--plan", PLAN, "--out", out],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return ("config generation failed", result.stderr)

    gs.insert("balsaConfigPath", out)
    return None
