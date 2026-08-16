{
  description = "litchee — async, builder-pattern Rust client for the Lichess API";

  # The toolchain pin lives in rust-projects, not here. Do not add a
  # rust-toolchain.toml: the dev shell materializes the shared one at the
  # project root so rustup and Nix agree, and there is only ever one pin to
  # bump. It carries the same 1.95.0 this crate declares as its MSRV in
  # Cargo.toml.
  inputs.rust-projects.url = "github:obazin/rust-projects";

  outputs =
    { rust-projects, ... }:
    rust-projects.forEachSystem (
      lib:
      lib.mkRustProject {
        name = "litchee";
        src = ./.;

        # For the dev shell: the native deps below, plus the release
        # automation `just` drives (see justfile) — git-cliff generates the
        # changelog and gh publishes the GitHub release. cargo-edit, which
        # bumps the version, and cargo-nextest come from the shared shell.
        extra = [
          lib.pkgs.cmake
          lib.pkgs.pkg-config
          lib.pkgs.just
          lib.pkgs.git-cliff
          lib.pkgs.gh
        ];

        # …and for the package/checks derivations: `aws-lc-sys` (the rustls
        # backend reqwest pulls in) builds its native library with CMake, and
        # pkg-config is needed by some -sys deps.
        nativeBuildInputs = [
          lib.pkgs.cmake
          lib.pkgs.pkg-config
        ];
      }
    );
}
