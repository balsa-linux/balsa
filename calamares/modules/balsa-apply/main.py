import libcalamares


TILING = {"niri", "hyprland", "mango", "i3", "sway", "bspwm", "xmonad", "dwm"}

LOGIN_AUTO = {
    "plasma6": "sddm",
    "gnome": "gdm",
    "cosmic": "cosmic-greeter",
}


def _resolve_login(choice, desktop):
    if choice != "auto":
        return choice
    if desktop in LOGIN_AUTO:
        return LOGIN_AUTO[desktop]
    return "ly" if desktop in TILING else "sddm"


def _resolve_bootloader(choice, firmware, filesystem):
    fallback = "systemd-boot" if firmware == "efi" else "grub"
    if choice == "auto":
        return fallback
    # cfggen rejects a plan that pairs Limine with anything but btrfs.
    if choice == "limine" and filesystem != "btrfs":
        libcalamares.utils.warning(
            "Limine needs btrfs, but the root filesystem is {!r}; using {}.".format(
                filesystem, fallback))
        return fallback
    if choice == "systemd-boot" and firmware != "efi":
        libcalamares.utils.warning("systemd-boot cannot boot legacy BIOS; using grub.")
        return "grub"
    return choice


def _resolve_specialisations(default, extras):
    # Standard is always shipped as a fallback unless it is already default.
    out = []
    if default != "standard":
        out.append("standard")
    for e in extras or []:
        # The introduction entry carries an empty id and is selectable like any other row.
        if e and e != default and e not in out:
            out.append(e)
    return out


def run():
    gs = libcalamares.globalstorage

    kernel = gs.value("packagechooser_kernel")
    desktop = gs.value("packagechooser_desktop")
    login = gs.value("packagechooser_loginmanager")
    bootloader = gs.value("packagechooser_bootloader")
    tuning = gs.value("packagechooser_tuning")
    extras = gs.value("packagechooser_tuningextra")

    if isinstance(extras, str):
        extras = [x for x in extras.split(",") if x]

    gs.insert("balsaKernel", kernel)
    gs.insert("balsaDesktop", desktop)
    gs.insert("balsaDesktopIsTiling", desktop in TILING)
    gs.insert("balsaLoginManager", _resolve_login(login, desktop))
    gs.insert(
        "balsaBootloader",
        _resolve_bootloader(bootloader, gs.value("balsaFirmware"), gs.value("balsaFilesystem")),
    )
    gs.insert("balsaTuningDefault", tuning)
    gs.insert("balsaTuningSpecialisations", _resolve_specialisations(tuning, extras))

    return None
