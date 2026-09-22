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


def _resolve_specialisations(default, extras):
    # Standard is always shipped as a fallback unless it is already default.
    out = []
    if default != "standard":
        out.append("standard")
    for e in extras or []:
        if e != default and e not in out:
            out.append(e)
    return out


def run():
    gs = libcalamares.globalstorage

    kernel = gs.value("packagechooser_kernel")
    desktop = gs.value("packagechooser_desktop")
    login = gs.value("packagechooser_loginmanager")
    tuning = gs.value("packagechooser_tuning")
    extras = gs.value("packagechooser_tuningextra")

    if isinstance(extras, str):
        extras = [x for x in extras.split(",") if x]

    gs.insert("balsaKernel", kernel)
    gs.insert("balsaDesktop", desktop)
    gs.insert("balsaDesktopIsTiling", desktop in TILING)
    gs.insert("balsaLoginManager", _resolve_login(login, desktop))
    gs.insert("balsaTuningDefault", tuning)
    gs.insert("balsaTuningSpecialisations", _resolve_specialisations(tuning, extras))

    return None
