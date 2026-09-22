{ stdenvNoCC, calamares-nixos-extensions }:

# calamares-nixos reads this package's XDG dirs, so an overlay swaps the installer config.
stdenvNoCC.mkDerivation {
  pname = "balsa-calamares";
  version = "0.1.0";

  src = ./.;

  installPhase = ''
    runHook preInstall

    mkdir -p $out/etc/calamares $out/lib/calamares/modules $out/share/calamares
    cp -r ${calamares-nixos-extensions}/etc/calamares/. $out/etc/calamares/
    cp -r ${calamares-nixos-extensions}/lib/calamares/modules/. $out/lib/calamares/modules/
    cp -r ${calamares-nixos-extensions}/share/calamares/. $out/share/calamares/
    chmod -R u+w $out

    # Upstream's locale.conf embeds a glibc-locales path from its own build.
    install -m644 settings.conf $out/etc/calamares/settings.conf
    install -m644 modules/*.conf $out/etc/calamares/modules/
    cp -r modules/balsa-* $out/lib/calamares/modules/
    cp -r branding/balsa $out/share/calamares/branding/

    substituteInPlace $out/etc/calamares/settings.conf \
      --replace-fail @modulesDir@ $out/lib/calamares/modules

    runHook postInstall
  '';

  dontBuild = true;
  dontFixup = true;
}
