import itertools
import re
import runpy
import sys
import types

warnings = []
lc = types.ModuleType("libcalamares")
setattr(lc, "utils", types.SimpleNamespace(warning=warnings.append))
sys.modules["libcalamares"] = lc
apply = runpy.run_path(sys.argv[1])

def ids(name):
    with open("{}/packagechooser_{}.conf".format(sys.argv[2], name)) as f:
        return re.findall(r'^- id: "(.*)"$', f.read(), re.M)

desktops = ids("desktop")
greeters = ids("loginmanager")
tunings = ids("tuning")
extras = ids("tuningextra")
assert len(desktops) == 10 and len(greeters) == 5, (desktops, greeters)
assert "" in extras, extras

auto = {"plasma6": "sddm", "gnome": "gdm", "cosmic": "cosmic-greeter"}
failures = []
for desktop, greeter in itertools.product(desktops, greeters):
    warnings.clear()
    got = apply["_resolve_login"](greeter, desktop)
    if greeter == "auto":
        want = auto.get(desktop, "sddm")
    elif greeter == "plasma-login-manager" and desktop != "plasma6":
        want = "sddm"
    else:
        want = greeter
    warned = greeter == "plasma-login-manager" and desktop != "plasma6"
    if got != want or bool(warnings) != warned:
        failures.append("login {}/{}: got {!r}, want {!r}, warnings {}".format(
            desktop, greeter, got, want, warnings))

subsets = [list(c) for n in range(len(extras) + 1) for c in itertools.combinations(extras, n)]
for default, chosen in itertools.product(tunings, subsets):
    got = apply["_resolve_specialisations"](default, chosen)
    want = (set(chosen) - {"", default}) | ({"standard"} - {default})
    if set(got) != want or len(got) != len(set(got)):
        failures.append("tuning {}/{}: got {!r}, want {!r}".format(default, chosen, got, want))

for f in failures:
    print(f)
assert not failures, "{} failures".format(len(failures))
print("PASS: {} login pairs, {} tuning pairs".format(
    len(desktops) * len(greeters), len(tunings) * len(subsets)))
