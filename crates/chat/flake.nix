{
  description = "Chat CLI - Unified Telegram and WhatsApp client";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Use nightly Rust (required for whatsapp-rust's portable_simd feature)
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };

        # Native dependencies required by the project
        nativeBuildInputs = with pkgs; [
          pkg-config
          rustToolchain
        ];

        buildInputs = with pkgs; [
          # SQLite for WhatsApp storage (Diesel)
          sqlite
          # OpenSSL for network connections
          openssl
          # Additional libraries that might be needed
          zlib
        ];

      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          # Environment variables
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;

          shellHook = ''
            echo "🦀 Chat CLI development environment"
            echo "Rust toolchain: nightly (required for whatsapp-rust)"
            echo ""
            echo "Available commands:"
            echo "  cargo check --features telegram,whatsapp  # Check both platforms"
            echo "  cargo build --release                      # Build with both platforms"
            echo "  cargo test                                 # Run tests"
            echo ""
            echo "Note: This project uses a patched grammers-session to avoid SQLite conflicts"
          '';
        };

        # Package definition
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "chat";
          version = "0.1.0";

          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              let
                baseName = baseNameOf path;
              in
              # Include whatsapp-rust even though it's not tracked by git
              (baseName == "whatsapp-rust" && type == "directory") ||
              (pkgs.lib.cleanSourceFilter path type);
          };

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs buildInputs;

          # Use nightly Rust for building
          RUSTC = "${rustToolchain}/bin/rustc";
          CARGO = "${rustToolchain}/bin/cargo";

          # Build with both telegram and whatsapp features
          buildFeatures = [ "telegram" "whatsapp" ];

          meta = with pkgs.lib; {
            description = "Unified CLI for Telegram and WhatsApp";
            homepage = "https://github.com/yourusername/chat";
            license = licenses.mit;
            maintainers = [ ];
          };
        };
      }
    );
}
