{ config, lib, modulesPath, pkgs, self, disko, ... }:

let
  system = pkgs.stdenv.hostPlatform.system;
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
  ];

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
