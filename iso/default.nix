{ modulesPath, pkgs, self, ... }:

{
  imports = [ "${modulesPath}/installer/cd-dvd/installation-cd-minimal.nix" ];

  environment.systemPackages = [ self.packages.${pkgs.stdenv.hostPlatform.system}.configgen ];
}
