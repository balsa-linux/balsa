{ config, lib, pkgs, ... }:

let
  # KDE Info Centre and friends resolve os-release's LOGO as an icon-theme name.
  logoIcon = pkgs.runCommand "balsa-logo-icon" { } ''
    install -Dm644 ${../../calamares/branding/balsa/logo.png} \
      $out/share/icons/hicolor/256x256/apps/balsa.png
  '';

  # Every desktop's menu button asks the icon theme for one of these names.
  menuIconNames = [
    "start-here"
    "start-here-symbolic"
    "start-here-kde"
    "start-here-kde-symbolic"
    "start-here-lxqt"
    "cinnamon-symbolic"
    "org.xfce.panel.applicationsmenu"
  ];

  desktop = {
    plasma = config.services.desktopManager.plasma6.enable;
    cinnamon = config.services.xserver.desktopManager.cinnamon.enable;
    mate = config.services.xserver.desktopManager.mate.enable;
    lxqt = config.services.xserver.desktopManager.lxqt.enable;
    xfce = config.services.xserver.desktopManager.xfce.enable;
  };

  # nixpkgs replaces the greeter's XDG_DATA_DIRS with just this, leaving it no icon theme.
  greeterDataDirs = lib.mkForce (
    "${config.services.displayManager.sessionData.desktops}/share:/run/current-system/sw/share"
  );

  # The Plasma greeter can be picked without the Plasma desktop, and it themes itself the same way.
  plasmaGreeter = config.services.displayManager.plasma-login-manager.enable;

  wayland =
    config.programs.niri.enable
    || config.programs.sway.enable
    || config.programs.hyprland.enable
    || config.programs.labwc.enable
    || config.programs.wayfire.enable;

  # Inheriting whatever the desktop already uses leaves every icon but the menu button alone.
  parentIconTheme =
    if desktop.plasma then "breeze-dark"
    else if desktop.lxqt then "breeze"
    else if desktop.mate then "menta"
    else if desktop.cinnamon then "gnome"
    else "Adwaita";

  iconTheme = pkgs.runCommand "balsa-icon-theme" { } ''
    dir=$out/share/icons/Balsa
    mkdir -p $dir/256x256/apps
    for name in ${lib.concatStringsSep " " menuIconNames}; do
      ln -s ${../../calamares/branding/balsa/logo.png} $dir/256x256/apps/$name.png
    done
    cat > $dir/index.theme <<EOF
    [Icon Theme]
    Name=Balsa
    Inherits=${parentIconTheme},Adwaita,hicolor
    Directories=256x256/apps

    [256x256/apps]
    Size=256
    MinSize=16
    MaxSize=512
    Type=Scalable
    Context=Applications
    EOF
  '';

  # A silent sed would leave the desktop on its own theme, so fail the build instead.
  retheme = name: source: script: pkgs.runCommand name { } ''
    sed ${script} ${source} > $out
    grep -q Balsa $out || { echo "${source} has no icon theme setting to replace"; exit 1; }
  '';

  wallpaper = ./balsawp.png;

  # Plasma keeps the wallpaper per user, so the layout script the first login runs sets it.
  plasmaLookAndFeel = pkgs.runCommand "balsa-look-and-feel" { nativeBuildInputs = [ pkgs.jq ]; } ''
    src=${pkgs.kdePackages.plasma-workspace}/share/plasma/look-and-feel/org.kde.breezedark.desktop
    dir=$out/share/plasma/look-and-feel/org.balsa.desktop
    mkdir -p $dir
    cp -r --no-preserve=mode $src/contents $dir/
    jq '.KPlugin.Id = "org.balsa.desktop" | .KPlugin.Name = "Balsa"' $src/metadata.json > $dir/metadata.json
    grep -q '^Theme=breeze-dark$' $dir/contents/defaults
    sed -i -e 's/^Theme=breeze-dark$/Theme=Balsa/' \
      -e 's|^Image=Next$|Image=${wallpaper}|' $dir/contents/defaults
    cat > $dir/contents/layouts/org.kde.plasma.desktop-layout.js <<EOF
    loadTemplate("org.kde.plasma.desktop.defaultPanel")

    var desktops = desktopsForActivity(currentActivity());
    for (var i = 0; i < desktops.length; i++) {
        desktops[i].wallpaperPlugin = "org.kde.image";
        desktops[i].currentConfigGroup = ["Wallpaper", "org.kde.image", "General"];
        desktops[i].writeConfig("Image", "file://${wallpaper}");
    }
    EOF
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
  # Upstream builds the boot label inline, so codeName can only be dropped by replacing the module.
  disabledModules = [ "system/activation/bootspec.nix" ];
  imports = [ ./bootspec.nix ];

  # A warning, not an assertion: this module is fetched from GitHub, so a failure here would block
  # nixos-rebuild on installed systems the moment they bump nixpkgs.
  warnings = lib.optional (
    builtins.hashFile "sha256" (pkgs.path + "/nixos/modules/system/activation/bootspec.nix")
    != "3ad576dc0e8e3a5c9df649b2abf77ad1b26dc6928e5eb3d2c811f46deb3a03b2"
  ) "nixpkgs bootspec.nix changed; re-diff modules/branding/bootspec.nix";

  environment.systemPackages =
    [ logoIcon iconTheme ]
    ++ lib.optional (desktop.plasma || plasmaGreeter) plasmaLookAndFeel
    # The greeter only sees /run/current-system/sw/share, so without the Plasma desktop it finds
    # no cursors (breeze), no wallpaper plugin (plasma-workspace) and no desktop theme
    # (libplasma). Missing the theme, it falls back to a light palette and brightens by 1.6.
    ++ lib.optionals (plasmaGreeter && !desktop.plasma) [
      pkgs.kdePackages.breeze
      pkgs.kdePackages.plasma-workspace
      pkgs.kdePackages.libplasma
    ];

  # Plasma, LXQt and Xfce read these as defaults; /etc/xdg comes first in XDG_CONFIG_DIRS.
  # Only the package name goes here: startplasma applies its defaults per user, so the
  # greeter, whose unit trims XDG_DATA_DIRS, keeps an icon theme it can actually find.
  environment.etc."xdg/kdeglobals" = lib.mkIf (desktop.plasma || plasmaGreeter) {
    # The greeter never runs startplasma, so it never expands the look-and-feel; its wallpaper
    # shader brightens by 1.6 unless the colours themselves say the scheme is dark.
    text = builtins.readFile
      "${pkgs.kdePackages.breeze}/share/color-schemes/BreezeDark.colors" + ''

      [KDE]
      LookAndFeelPackage=org.balsa.desktop
    '';
  };

  environment.etc."xdg/lxqt/lxqt.conf" = lib.mkIf desktop.lxqt {
    source = retheme "balsa-lxqt.conf" "${pkgs.lxqt.lxqt-session}/share/lxqt/lxqt.conf"
      "'s/^icon_theme=.*/icon_theme=Balsa/'";
  };

  # pcmanfm-qt paints the LXQt desktop and keeps its own icon theme name.
  environment.etc."xdg/pcmanfm-qt/lxqt/settings.conf" = lib.mkIf desktop.lxqt {
    source = retheme "balsa-pcmanfm-qt.conf"
      "${pkgs.lxqt.pcmanfm-qt}/share/pcmanfm-qt/lxqt/settings.conf"
      "-e 's|^Wallpaper=.*|Wallpaper=${wallpaper}|' -e 's/^IconThemeName=.*/IconThemeName=Balsa/'";
  };

  # xfdesktop has no config until the user picks a wallpaper; its fallback is a build flag.
  nixpkgs.overlays = [
    (final: prev: {
      xfdesktop = prev.xfdesktop.overrideAttrs (old: {
        configureFlags = (old.configureFlags or [ ])
          ++ [ "--with-default-backdrop-filename=${wallpaper}" ];
      });
    })
  ];

  environment.etc."xdg/xfce4/xfconf/xfce-perchannel-xml/xsettings.xml" = lib.mkIf desktop.xfce {
    source = retheme "balsa-xsettings.xml"
      "${pkgs.xfce.xfce4-settings}/etc/xdg/xfce4/xfconf/xfce-perchannel-xml/xsettings.xml"
      "'/IconThemeName/s/value=\"[^\"]*\"/value=\"Balsa\"/'";
  };

  systemd.services.plasmalogin.environment.XDG_DATA_DIRS =
    lib.mkIf config.services.displayManager.plasma-login-manager.enable greeterDataDirs;
  systemd.user.services.plasma-login.environment.XDG_DATA_DIRS =
    lib.mkIf config.services.displayManager.plasma-login-manager.enable greeterDataDirs;

  # Keys for a desktop that is not installed are inert, so all three schemas get set.
  programs.dconf.profiles.user.databases = [
    {
      settings = {
        "org/gnome/desktop/interface" = {
          icon-theme = "Balsa";
          gtk-theme = "Adwaita-dark";
          color-scheme = "prefer-dark";
        };
        "org/cinnamon/desktop/interface" = {
          icon-theme = "Balsa";
          gtk-theme = "Adwaita-dark";
        };
        "org/mate/desktop/interface" = {
          icon-theme = "Balsa";
          gtk-theme = "Adwaita-dark";
        };
        "org/gnome/desktop/background" = {
          picture-uri = "file://${wallpaper}";
          picture-uri-dark = "file://${wallpaper}";
          picture-options = "zoom";
        };
        "org/cinnamon/desktop/background" = {
          picture-uri = "file://${wallpaper}";
          picture-options = "zoom";
        };
        "org/mate/desktop/background" = {
          picture-filename = "${wallpaper}";
          picture-options = "zoom";
        };
      };
    }
  ];

  # Bare window managers paint no desktop of their own; every X session gets the wallpaper here.
  services.xserver.displayManager.sessionCommands =
    lib.mkIf config.services.xserver.enable "${pkgs.feh}/bin/feh --no-fehbg --bg-fill ${wallpaper}";

  # The Wayland compositors have no desktop either, and each starts graphical-session.target.
  systemd.user.services.balsa-wallpaper = lib.mkIf wayland {
    description = "Balsa wallpaper";
    partOf = [ "graphical-session.target" ];
    after = [ "graphical-session.target" ];
    wantedBy = [ "graphical-session.target" ];
    serviceConfig.ExecStart = "${pkgs.swaybg}/bin/swaybg --mode fill --image ${wallpaper}";
  };

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

  # nixpkgs points Limine at a nixos-artwork wallpaper; the option exists even when Limine is off.
  boot.loader.limine.style.wallpapers = [ wallpaper ];

  boot.consoleLogLevel = lib.mkIf quietBoot 3;
  boot.initrd.verbose = lib.mkIf quietBoot false;
  boot.kernelParams = lib.mkIf quietBoot [ "quiet" "udev.log_level=3" "systemd.show_status=auto" ];
}
