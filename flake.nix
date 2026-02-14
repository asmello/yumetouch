{
  description = "macOS YubiKey touch notifier daemon";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      ...
    }:
    let
      supportedSystems = [ "aarch64-darwin" ];
      forEachSystem = nixpkgs.lib.genAttrs supportedSystems;

      mkPkgs =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

      mkCraneLib =
        system:
        let
          pkgs = mkPkgs system;
          toolchain = pkgs.rust-bin.stable.latest.default;
        in
        (crane.mkLib pkgs).overrideToolchain toolchain;

      src =
        system:
        let
          craneLib = mkCraneLib system;
        in
        nixpkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter =
            path: type:
            (craneLib.filterCargoSources path type) || (builtins.match ".*resources/.*" path != null);
        };
    in
    {
      packages = forEachSystem (
        system:
        let
          craneLib = mkCraneLib system;
          commonArgs = {
            src = src system;
            strictDeps = true;
          };
        in
        {
          default = craneLib.buildPackage (
            commonArgs
            // {
              cargoArtifacts = craneLib.buildDepsOnly commonArgs;
              meta = {
                description = "macOS YubiKey touch notifier daemon";
                license = nixpkgs.lib.licenses.mit;
                platforms = [ "aarch64-darwin" ];
              };
            }
          );
        }
      );

      overlays.default = final: _prev: {
        yumetouch = self.packages.${final.system}.default;
      };

      formatter = forEachSystem (system: (mkPkgs system).nixfmt-tree);

      devShells = forEachSystem (
        system:
        let
          pkgs = mkPkgs system;
          toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "clippy"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = [
              toolchain
            ];
          };
        }
      );
    };
}
