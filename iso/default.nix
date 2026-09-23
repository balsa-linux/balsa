{ config, lib, modulesPath, pkgs, self, disko, ... }:

let
  system = pkgs.stdenv.hostPlatform.system;
  artwork = ../calamares/branding/balsa/welcome.png;

  # NixOS's GRUB theme recoloured to the artwork's palette: #1A1A1A ground, #E9C6AF tan.
  grubTheme = pkgs.runCommand "balsa-grub-theme" { nativeBuildInputs = [ pkgs.imagemagick ]; } ''
    cp -r --no-preserve=mode ${pkgs.nixos-grub2-theme} $out
    cd $out
    # PNG32: GRUB only decodes 8-bit RGB(A), and ImageMagick writes flat colours as grey or palette.
    magick ${artwork} -fuzz 4% -trim +repage -fuzz 6% -transparent "#1A1A1A" \
      -resize 319x100 -background none -gravity center -extent 319x100 PNG32:logo.png
    magick -size 1x1 xc:"#1A1A1A" PNG32:background.png
    for f in boot_menu_*.png; do magick "$f" -fill "#1A1A1A" -colorize 100 "PNG32:$f"; done
    magick select_c.png -fill "#2A2A2A" -colorize 100 PNG32:select_c.png
    for f in icons/*.png; do magick "$f" -fill "#E6E6E6" -colorize 100 "PNG32:$f"; done
    sed -i -e 's/item_color = "#232627"/item_color = "#E6E6E6"/' \
      -e 's/selected_item_color= "#232627"/selected_item_color= "#E9C6AF"/' \
      -e 's/border_color = #5579C4/border_color = #E9C6AF/' \
      -e 's/bg_color = #7EBAE4/bg_color = #262626/' \
      -e 's/fg_color = #5579C4/fg_color = #E9C6AF/' theme.txt
  '';

  # i3 -C rejects a bad config at build time; i3 only shows a nagbar at runtime.
  i3Config = pkgs.runCommand "balsa-i3-config" { nativeBuildInputs = [ pkgs.i3 ]; } ''
    cat > $out <<"CONFIG"
    set $mod Mod4
    font pango:DejaVu Sans Mono 10
    bindsym $mod+Return exec alacritty
    bindsym $mod+d exec dmenu_run
    bindsym $mod+Shift+q kill
    bindsym $mod+Shift+e exit
    floating_modifier $mod
    bar {
        status_command i3status
    }
    # i3 has no XDG autostart; pkexec matches what the packaged desktop entry does.
    exec --no-startup-id pkexec calamares
    CONFIG
    i3 -C -c $out
  '';

  # syslinux draws its menu in the top rows and the timeout at the bottom, so the lockup sits centred.
  biosSplash = pkgs.runCommand "balsa-bios-splash.png" { nativeBuildInputs = [ pkgs.imagemagick ]; } ''
    magick -size 800x600 xc:"#1A1A1A" \
      \( ${artwork} -fuzz 4% -trim +repage -resize 420x \) \
      -gravity center -composite PNG32:$out
  '';
in
{
  imports = [ "${modulesPath}/installer/cd-dvd/installation-cd-graphical-calamares.nix" ];

  services.xserver.windowManager.i3.enable = true;
  services.xserver.windowManager.i3.configFile = i3Config;

  services.xserver.displayManager.lightdm.enable = true;
  services.displayManager.autoLogin = {
    enable = true;
    user = "nixos";
  };
  services.displayManager.defaultSession = "none+i3";

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
    pkgs.alacritty
  ];

  # Calamares and i3 fall back to bitmap fonts without a real sans family installed.
  fonts.packages = [ pkgs.noto-fonts ];

  # Boot menu reads "Balsa 27.0a Installer"; the ISO module sets baseName at normal priority.
  system.nixos.label = "27.0a";
  image.baseName = lib.mkForce "balsa-27.0a-x86_64";
  isoImage.volumeID = "balsa-27.0a-x86_64";
  isoImage.grubTheme = grubTheme;
  isoImage.splashImage = biosSplash;
  # nixpkgs' default with its light colours swapped for the artwork's; FG/BG are AARRGGBB.
  isoImage.syslinuxTheme = ''
    MENU TITLE ${config.system.nixos.distroName}
    MENU RESOLUTION 800 600
    MENU CLEAR
    MENU ROWS 6
    MENU CMDLINEROW -4
    MENU TIMEOUTROW -3
    MENU TABMSGROW  -2
    MENU HELPMSGROW -1
    MENU HELPMSGENDROW -1
    MENU MARGIN 0
    MENU COLOR BORDER       30;44      #00000000    #00000000   none
    MENU COLOR SCREEN       37;40      #FFE6E6E6    #001A1A1A   none
    MENU COLOR TABMSG       31;40      #80E6E6E6    #00000000   none
    MENU COLOR TIMEOUT      1;37;40    #FFE9C6AF    #00000000   none
    MENU COLOR TIMEOUT_MSG  37;40      #FFE6E6E6    #00000000   none
    MENU COLOR CMDMARK      1;36;40    #FFE9C6AF    #00000000   none
    MENU COLOR CMDLINE      37;40      #FFE6E6E6    #00000000   none
    MENU COLOR TITLE        1;36;44    #00000000    #00000000   none
    MENU COLOR UNSEL        37;44      #FFE6E6E6    #00000000   none
    MENU COLOR SEL          7;37;40    #FFE9C6AF    #FF2A2A2A   std
  '';

  # balsa-config-export pins installed systems to this, the nixpkgs of the live store.
  environment.etc."balsa/nixpkgs-rev".text = config.system.nixos.revision;

  # Installed systems fetch nixosModules.branding from GitHub at this commit.
  environment.etc."balsa/balsa-rev".text = self.rev or (lib.removeSuffix "-dirty" self.dirtyRev);

  nix.settings = {
    experimental-features = [ "nix-command" "flakes" ];
    # The installer downloads a full system closure; the defaults gave up too easily.
    http-connections = 10;
    download-attempts = 10;
  };

  system.stateVersion = "26.05";
}
