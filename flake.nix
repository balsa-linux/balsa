{
  description = "balsa - lightweight, malleable linux based on Nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    disko.url = "github:nix-community/disko/latest";
    disko.inputs.nixpkgs.follows = "nixpkgs";
    cachyos-kernel.url = "github:xddxdd/nix-cachyos-kernel/release";
  };

  outputs = { self, nixpkgs, disko, cachyos-kernel, ... }: {
    nixosConfigurations.iso = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      specialArgs = { inherit self; };
      modules = [
        ./iso/default.nix
        disko.nixosModules.disko
      ];
    };

    packages.x86_64-linux.configgen =
      (nixpkgs.legacyPackages.x86_64-linux.callPackage ./cfggen {});
  };
}
