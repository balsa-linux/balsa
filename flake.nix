{
  description = "balsa - lightweight, malleable linux based on Nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    disko.url = "github:nix-community/disko/latest";
    disko.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, disko, ... }: {
    nixosConfigurations.iso = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      specialArgs = { inherit self disko; };
      modules = [
        ./iso/default.nix
        disko.nixosModules.disko
        self.nixosModules.branding
      ];
    };

    # Installed systems import these through a balsa input in their generated flake.
    nixosModules.branding = ./modules/branding;

    packages.x86_64-linux.configgen =
      (nixpkgs.legacyPackages.x86_64-linux.callPackage ./cfggen {});

    checks.x86_64-linux.parent-disk =
      import ./tests/parent-disk.nix nixpkgs.legacyPackages.x86_64-linux;

    checks.x86_64-linux.apply-resolution =
      import ./tests/apply-resolution.nix nixpkgs.legacyPackages.x86_64-linux;

    checks.x86_64-linux.check-bundle =
      import ./tests/check-bundle.nix nixpkgs.legacyPackages.x86_64-linux;
  };
}
