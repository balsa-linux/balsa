import os
import subprocess
import libcalamares


def _cpu_vendor():
    try:
        with open("/proc/cpuinfo") as f:
            text = f.read()
    except OSError:
        return "unknown"
    if "GenuineIntel" in text:
        return "intel"
    if "AuthenticAMD" in text:
        return "amd"
    return "unknown"


def _gpu_vendors():
    out = subprocess.run(["lspci", "-nn"], capture_output=True, text=True).stdout
    vendors = set()
    for line in out.splitlines():
        low = line.lower()
        if "vga" not in low and "3d controller" not in low:
            continue
        if "nvidia" in low:
            vendors.add("nvidia")
        elif "amd" in low or "ati" in low:
            vendors.add("amd")
        elif "intel" in low:
            vendors.add("intel")
    return sorted(vendors)


def _firmware_type():
    return "uefi" if os.path.isdir("/sys/firmware/efi") else "bios"


def run():
    gs = libcalamares.globalstorage
    # An empty GPU list would silently disable the Nvidia driver.
    try:
        gpus = _gpu_vendors()
    except FileNotFoundError:
        return ("Hardware detection failed",
                "balsa-hardware-detect could not run lspci; GPU vendors are unknown.")
    gs.insert("balsaCpuVendor", _cpu_vendor())
    gs.insert("balsaGpuVendors", gpus)
    gs.insert("balsaFirmware", _firmware_type())
    return None
