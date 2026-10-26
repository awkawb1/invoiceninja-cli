{
  description = "invoiceninja-cli: a command-line client for the Invoice Ninja v5 API";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    let
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

      # Shared between `packages.default` (built below with the pinned
      # rust-overlay toolchain, for parity with `nix develop`) and
      # `overlays.default` (built with whatever `rustPlatform` the
      # *consuming* nixpkgs provides, per the usual nixpkgs packaging
      # convention — see the note on `overlays.default` below). Takes its
      # inputs as a plain attrset so both call sites, and `callPackage`,
      # can hand it the pieces however they're sourced.
      mkInvoiceninjaCli = { lib, rustPlatform, pkg-config, openssl }:
        rustPlatform.buildRustPackage {
          pname = "invoiceninja-cli";
          version = cargoToml.package.version;
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [ pkg-config ];
          buildInputs = [ openssl ];

          meta = {
            description = cargoToml.package.description;
            license = lib.licenses.mit;
            mainProgram = "invoiceninja-cli";
            platforms = lib.platforms.unix;
          };
        };
    in
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Kept in step with invoiceninjars-sdk's own `devVersion`, so the two
        # sibling crates build with the same toolchain during development.
        devVersion = "1.99.1";

        rustToolchain = pkgs.rust-bin.stable.${devVersion}.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };

        # Built with the pinned dev toolchain, not nixpkgs' own rustc, so a
        # `nix build` matches what `nix develop` compiles with.
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [ rustToolchain pkgs.pkg-config pkgs.openssl ];
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        # Unlike invoiceninjars-sdk, this crate has a `[[bin]]` target, so
        # `nix build` produces an installable binary, not just a test gate.
        # `invoiceninja-sdk` resolves from crates.io (see Cargo.toml), so
        # unlike before this no longer needs a `../invoiceninjars-sdk`
        # checkout to be present in the sandbox.
        packages.default = mkInvoiceninjaCli {
          inherit (pkgs) lib pkg-config openssl;
          inherit rustPlatform;
        };
      }) // {
      # For a flake that wants `invoiceninja-cli` in its own `pkgs`, not for
      # this flake's own `packages.default`/`devShells.default` above (those
      # keep the pinned rust-overlay toolchain). `callPackage` resolves
      # `rustPlatform` from the *consuming* nixpkgs, which means it builds
      # with whatever rustc that channel ships — this crate's `rust-version
      # = "1.88"` in Cargo.toml has to be met there too, same as any other
      # nixpkgs Rust package; this overlay doesn't carry the rust-overlay
      # pin along with it.
      overlays.default = final: _prev: {
        invoiceninja-cli = final.callPackage mkInvoiceninjaCli { };
      };
    };
}
