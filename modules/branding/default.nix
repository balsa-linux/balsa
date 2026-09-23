{ config, lib, pkgs, ... }:

let
  # KDE Info Centre and friends resolve os-release's LOGO as an icon-theme name.
  logoIcon = pkgs.runCommand "balsa-logo-icon" { } ''
    install -Dm644 ${../../calamares/branding/balsa/logo.png} \
      $out/share/icons/hicolor/256x256/apps/balsa.png
  '';

  # bgrt's ImageDir points into the plymouth package, which ships no watermark.
  plymouthTheme =
    pkgs.runCommand "balsa-plymouth-theme" { nativeBuildInputs = [ pkgs.imagemagick ]; }
      ''
        themes=${config.boot.plymouth.package}/share/plymouth/themes
        dir=$out/share/plymouth/themes/balsa
        mkdir -p $dir
        cp $themes/spinner/* $dir/
        sed "s,^ImageDir=.*,ImageDir=$dir," $themes/bgrt/bgrt.plymouth > $dir/balsa.plymouth
        magick ${./balsa-horiz.png} -trim +repage -resize 256x PNG32:$dir/watermark.png
      '';

  # The installer ISO keeps its boot messages; they are how failed installs get diagnosed.
  quietBoot = config.system.nixos.variant_id != "installer";
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

  # Draws the firmware's own logo, with the Balsa lockup as the watermark beneath it.
  boot.plymouth = {
    enable = true;
    theme = "balsa";
    themePackages = [ plymouthTheme ];
  };

  boot.consoleLogLevel = lib.mkIf quietBoot 3;
  boot.initrd.verbose = lib.mkIf quietBoot false;
  boot.kernelParams = lib.mkIf quietBoot [ "quiet" "udev.log_level=3" "systemd.show_status=auto" ];
}
