{ config, lib, pkgs, ... }:

let
  # KDE Info Centre and friends resolve os-release's LOGO as an icon-theme name.
  logoIcon = pkgs.runCommand "balsa-logo-icon" { } ''
    install -Dm644 ${../../calamares/branding/balsa/logo.png} \
      $out/share/icons/hicolor/256x256/apps/balsa.png
  '';
in
{
  environment.systemPackages = [ logoIcon ];

  # A non-nixos distroId drops nixos.org's URLs; systemd-boot, GRUB and Limine title entries with distroName.
  system.nixos = {
    distroId = "balsa";
    distroName = "Balsa";
    vendorId = "balsa";
    vendorName = "Balsa";
    extraOSReleaseArgs = {
      VERSION = "27.0a";
      VERSION_ID = "27.0a";
      VERSION_CODENAME = "27";
      CPE_NAME = "cpe:/o:balsa:balsa:27.0a";
      PRETTY_NAME = "Balsa 27.0a";
      LOGO = "balsa";
      HOME_URL = "https://balsa.aylah.dev";
      SUPPORT_URL = "https://github.com/balsa-linux/balsa/issues";
      BUG_REPORT_URL = "https://github.com/balsa-linux/balsa/issues";
    };
  };

  # The gdm package sets the NixOS snowflake as its own gsettings default.
  programs.dconf.profiles.gdm.databases = lib.mkIf config.services.displayManager.gdm.enable [
    {
      settings."org/gnome/login-screen".logo =
        "${logoIcon}/share/icons/hicolor/256x256/apps/balsa.png";
    }
  ];
}
