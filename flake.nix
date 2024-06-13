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
            config.allowBroken = true;
          };

          nativeBuildInputs = with pkgs; [ protobuf3_24 ];

          buildInputs = with pkgs; [ pkg-config openssl ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          packages = with pkgs; [
            cargo-llvm-cov
          ];

          defaultShell = with pkgs; mkShell {
            inherit buildInputs packages;

            nativeBuildInputs = with pkgs; nativeBuildInputs ++ [
              rust-bin.nightly.latest.default
            ];

            PROTOC = "${protobuf3_24}/bin/protoc";
          };
        in
        with pkgs;
        {
          devShells = rec {
            default = nightly;

            nightly = with pkgs; mkShell {
              inherit buildInputs packages;

              nativeBuildInputs = with pkgs; nativeBuildInputs ++ [
                rust-bin.nightly.latest.default
              ];

              PROTOC = "${protobuf3_24}/bin/protoc";
            };

            stable = with pkgs; mkShell {
              name = "stable";

              inherit buildInputs packages;

              nativeBuildInputs = with pkgs; nativeBuildInputs ++ [
                rust-bin.stable.latest.default
              ];

              PROTOC = "${protobuf3_24}/bin/protoc";
            };
          };
        }
      );
}
