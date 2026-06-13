{
  description = "litchee — devShell composed from chess-flake bundles";

  inputs.workspace.url = "github:obazin/chess-flake";

  outputs =
    { self, workspace }:
    {
      devShells = builtins.mapAttrs (system: lib: {
        default = lib.bundles.rustShell { name = "litchee"; };
      }) workspace.lib;
    };
}
