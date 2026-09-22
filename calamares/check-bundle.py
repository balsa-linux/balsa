#!/usr/bin/env python3

import os
import sys

import yaml

CONFIG_NEEDED = {"welcome", "partition", "mount", "unpackfs", "users",
                 "finished", "packagechooser", "locale", "keyboard"}


def main(argv):
    cfgdir = os.path.abspath(argv[1] if len(argv) > 1 else os.path.dirname(__file__) or ".")
    settings = yaml.safe_load(open(os.path.join(cfgdir, "settings.conf")))

    search = [os.path.join(cfgdir, "modules")]
    for extra in argv[2:]:
        search += [os.path.join(extra, "modules"), extra]
    instances = {}
    for inst in settings.get("instances") or []:
        key = "%s@%s" % (inst["module"], inst.get("id", inst["module"]))
        instances[key] = (inst["module"], inst.get("config", inst["module"] + ".conf"))

    errors = []
    for phase in settings["sequence"]:
        for entry in next(iter(phase.values())):
            module, conf = instances.get(entry, (entry.split("@")[0], entry.split("@")[0] + ".conf"))
            if "@" in entry and entry not in instances:
                errors.append("%s: no instance declared" % entry)
            found = [d for d in search if os.path.isfile(os.path.join(d, module, "module.desc"))]
            if not found and module.startswith("balsa-"):
                errors.append("%s: no module.desc in %s" % (module, search))
            if module in CONFIG_NEEDED and not any(
                    os.path.isfile(os.path.join(d, conf)) for d in search):
                errors.append("%s: missing %s" % (entry, conf))

    for e in errors:
        print("ERROR", e)
    print("%d sequence errors" % len(errors))
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
