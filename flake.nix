{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];

          pkgs = import nixpkgs {
            inherit system overlays;
          };

          nativeBuildInputs = with pkgs; [ rust-bin.nightly.latest.default protobuf3_24 ];

          buildInputs = with pkgs; [ pkg-config openssl ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];
        in
        with pkgs;
        {
          devShells.default = with pkgs; mkShell {
            inherit nativeBuildInputs buildInputs;

            PROTOC = "${protobuf3_24}/bin/protoc";
          };
        }
      );
}
