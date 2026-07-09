{
  description = "litchee — async, builder-pattern Rust client for the Lichess API";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        # The exact Rust toolchain pinned in `rust-toolchain.toml` (1.95.0 —
        # the crate's MSRV and the README's stated version). `fromToolchainFile`
        # verifies the channel manifest against `sha256`, so the toolchain is
        # fully deterministic and never drifts with `nix flake update`. The hash
        # is of the channel manifest (platform-independent), so one value works
        # across all systems.
        toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-gh/xTkxKHL4eiRXzWv8KP7vfjSk61Iq48x47BEDFgfk=";
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            toolchain
            # `aws-lc-sys` (the rustls backend reqwest pulls in) builds its
            # native library with CMake; pkg-config is needed by some -sys deps.
            pkgs.cmake
            pkgs.pkg-config
            # Release automation driven by `just` (see justfile):
            # git-cliff generates the changelog, cargo-edit bumps the version,
            # and gh publishes the GitHub release.
            pkgs.just
            pkgs.git-cliff
            pkgs.cargo-edit
            pkgs.gh
          ];

          # rust-analyzer resolves std sources through this path.
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
