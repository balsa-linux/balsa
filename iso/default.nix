{ config, lib, modulesPath, pkgs, self, disko, ... }:

let
  system = pkgs.stdenv.hostPlatform.system;

  # KDE Info Centre and friends read os-release's LOGO as an icon-theme name.
  logoIcon = pkgs.runCommand "balsa-logo-icon" { } ''
    install -Dm644 ${../calamares/branding/balsa/logo.png} \
      $out/share/icons/hicolor/256x256/apps/balsa.png
  '';
in
{
  imports = [ "${modulesPath}/installer/cd-dvd/installation-cd-graphical-calamares-plasma6.nix" ];

  # calamares-nixos wraps calamares with this package's XDG dirs, so this overlay makes it the Balsa installer.
  nixpkgs.overlays = [
    (final: prev: {
      calamares-nixos-extensions = final.callPackage ../calamares {
        calamares-nixos-extensions = prev.calamares-nixos-extensions;
      };
    })
  ];

  environment.systemPackages = [
    self.packages.${system}.configgen
    disko.packages.${system}.disko
    pkgs.mkpasswd
    logoIcon
  ];

  # A non-nixos distroId drops the nixos.org URLs from os-release.
  system.nixos.distroId = "balsa";
  system.nixos.distroName = "Balsa";
  system.nixos.vendorId = "balsa";
  system.nixos.vendorName = "Balsa";
  system.nixos.extraOSReleaseArgs = {
    VERSION = "27.0a";
    VERSION_ID = "27.0a";
    VERSION_CODENAME = "27";
    CPE_NAME = "cpe:/o:balsa:balsa:27.0a";
    PRETTY_NAME = "Balsa 27.0a";
    LOGO = "balsa";
    HOME_URL = "https://balsa.aylah.dev";
    SUPPORT_URL = "https://github.com/aylah/balsa/issues";
    BUG_REPORT_URL = "https://github.com/aylah/balsa/issues";
  };

  # balsa-config-export pins installed systems to this, the nixpkgs of the live store.
  environment.etc."balsa/nixpkgs-rev".text = config.system.nixos.revision;

  nix.settings = {
    experimental-features = [ "nix-command" "flakes" ];
    # The installer downloads a full system closure; the defaults gave up too easily.
    http-connections = 10;
    download-attempts = 10;
  };

  system.stateVersion = "26.05";
}
