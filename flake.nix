{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    rust-overlay.inputs.flake-utils.follows = "flake-utils";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };

          protobuf = pkgs.protobuf3_24;
        in
        with pkgs;
        {
          devShells.default = with pkgs; mkShell {
            packages = [
              niv
              nixpkgs-fmt
              pkg-config
              openssl
              rust-bin.nightly.latest.default
              protobuf
            ] ++ lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.SystemConfiguration
            ];

            PROTOC = "${protobuf}/bin/protoc";
          };
        }
      );
}
