{
  description = "invoiceninja-cli: a command-line client for the Invoice Ninja v5 API";

  # Mirrors invoiceninja-nixos's own nixConfig: without this, building
  # `nixosConfigurations.invoiceninja-test-*` here rebuilds the whole PHP/UI
  # app (and its chromium-headless-shell closure) from source instead of
  # pulling the cached result.
  nixConfig = {
    extra-substituters = [ "https://invoiceninja-nixos.cachix.org" ];
    extra-trusted-public-keys = [
      "invoiceninja-nixos.cachix.org-1:TmRZaji8FFVjjIw9KZmvLrxMxfP+JDThOyOX04z526A="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    # For a real Invoice Ninja instance to test the CLI against — see
    # CLAUDE.md's "Testing against a real server". Not `follows`-ed to this
    # flake's own `nixpkgs`: it pins its own (nixos-26.05, as of writing),
    # and `nixosConfigurations.test-nginx`/`test-caddy` below are consumed
    # as already-built flake outputs, not re-evaluated against our
    # nixos-unstable — the two never need to agree.
    invoiceninja-nixos.url = "git+ssh://gitea@gitea.awkawb.cloud/awkawb/invoiceninja-nixos.git";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, invoiceninja-nixos, ... }:
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
          # jq is for parsing the /api/v1/login response when testing
          # against a real server — see CLAUDE.md's "Testing against a
          # real server".
          packages = [ rustToolchain pkgs.pkg-config pkgs.openssl pkgs.jq ];
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

        # Automated: boots a real Invoice Ninja instance (invoiceninja-nixos's
        # module, not a mock) and runs actual `invoiceninja-cli` commands
        # against it — create/list/get/update/delete across clients,
        # invoices, and payments, including that `update` preserves fields
        # it wasn't told about. `tests/` is a vendored, adapted copy of
        # invoiceninja-nixos's own test scaffolding; see its
        # nixos-test.nix for what it does and doesn't check (it
        # deliberately skips things like PDF generation and the language-
        # list regression, which are that repo's own concern, not this
        # CLI's). Uses invoiceninja-nixos's own pinned nixpkgs (26.05), not
        # this flake's nixos-unstable, to build+evaluate the VM/module —
        # matches what that module was actually tested against. The CLI
        # binary itself (`self.packages.${system}.default`, built against
        # our own nixos-unstable) runs fine inside it regardless: Nix
        # binaries carry their own closure, so differing nixpkgs revisions
        # in the same system closure is an explicitly supported case, not a
        # version mismatch to work around.
        checks.invoiceninja-cli = import ./tests/nixos-test.nix {
          nixpkgsPath = invoiceninja-nixos.inputs.nixpkgs;
          inherit system;
          invoiceninjaModule = invoiceninja-nixos.nixosModules.invoiceninja;
          invoiceninjaCliPkg = self.packages.${system}.default;
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

      # Real Invoice Ninja instances to test this CLI against, reusing
      # invoiceninja-nixos's own test VMs as-is rather than redefining them
      # here — same principle as this crate being a client of the SDK, not
      # a second implementation of it. Known test credentials baked into
      # that repo's tests/common.nix: admin@email.com / password (from
      # tests/invoice_ninja_test_password). See CLAUDE.md for how to boot
      # one and get an API token from it.
      nixosConfigurations = {
        invoiceninja-test-nginx = invoiceninja-nixos.nixosConfigurations.test-nginx;
        invoiceninja-test-caddy = invoiceninja-nixos.nixosConfigurations.test-caddy;
      };
    };
}
