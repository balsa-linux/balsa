# Checked against the built bundle, since that is the merge of upstream's config and ours that the ISO ships.
pkgs:

let
  bundle = pkgs.callPackage ../calamares { };
  python = pkgs.python3.withPackages (ps: [ ps.pyyaml ]);
in
pkgs.runCommand "balsa-check-bundle" { } ''
  ${python.interpreter} ${../calamares/check-bundle.py} ${bundle}/etc/calamares ${bundle}/lib/calamares | tee $out
''
