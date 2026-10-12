# balsa-apply resolves impossible greeter and tuning combinations after disko has already written the disk, so every reachable pair is checked.
pkgs:

pkgs.runCommand "balsa-apply-resolution" { } ''
  ${pkgs.python3.interpreter} ${./apply-resolution.py} ${../calamares/modules/balsa-apply/main.py} ${../calamares/modules} | tee $out
''
