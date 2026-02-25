{
  description = "Strudel - Live coding patterns on the web";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Node.js version 22 as specified in .nvmrc
        nodejs = pkgs.nodejs_22;

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Core development tools
            nodejs
            nodePackages.pnpm

            # Build and development tools
            git

            # Rust toolchain for Tauri desktop and CLAP plugin
            cargo
            rustc
            rustfmt
            clippy
            pkg-config

            # Tauri desktop dependencies
            openssl
            webkitgtk
            gtk3
            libsoup

            # CLAP plugin audio dependencies
            alsa-lib
            libjack2
          ];

          shellHook = ''
            echo "🌀 Strudel Development Environment 🌀"
            echo ""
            echo "Node.js version: $(node --version)"
            echo "pnpm version: $(pnpm --version)"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo ""
            echo "Available commands:"
            echo "  pnpm i          - Install dependencies"
            echo "  pnpm dev        - Start development server"
            echo "  pnpm test       - Run tests"
            echo "  pnpm lint       - Run linter"
            echo "  pnpm build      - Build for production"
            echo ""
            echo "CLAP Plugin Development:"
            echo "  cd packages/clap-plugin"
            echo "  cargo build              - Build debug plugin"
            echo "  cargo build --release    - Build release plugin"
            echo "  cargo clippy             - Lint Rust code"
            echo ""

            # Set up pnpm store directory in the project to avoid global pollution
            export PNPM_HOME="$PWD/.pnpm-store"
            export PATH="$PNPM_HOME:$PATH"

            # Ensure node_modules/.bin is in PATH
            export PATH="$PWD/node_modules/.bin:$PATH"
          '';
        };
      }
    );
}
