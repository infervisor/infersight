{
  description = "InferSight — GPU & system monitoring suite";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "infersight";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ ];

          # Only build the unified CLI binary
          cargoBuildFlags = [ "--package" "is-cli" ];

          meta = {
            description = "Unified InferSight CLI — export, monitor, and control GPUs";
            homepage = "https://github.com/infervisor/infersight";
          };
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/infersight";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo
            pkgs.rustc
            pkgs.rustfmt
            pkgs.clippy
            pkgs.rust-analyzer
            pkgs.pkg-config
          ];
        };
      }
    );
}
