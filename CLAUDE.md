# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A command-line client for the [Invoice Ninja](https://www.invoiceninja.com/)
v5 API, built on [`invoiceninja-sdk`](https://crates.io/crates/invoiceninja-sdk),
consumed as a normal crates.io dependency (not a path dependency on a sibling
checkout — see "Development environment" below for how that changed).
That crate's own `GOALS.md` (at `../invoiceninjars-sdk/GOALS.md`, if you have
a checkout for reference) explains why it's a separate crate rather than a
feature of the library — dependencies, MSRV, and versioning, all pre-1.0.
Read that `GOALS.md` before making a design call here; this project exists
downstream of decisions made there.

**This project's own [`GOALS.md`](GOALS.md) states what it is for, what it
deliberately is not, and what is still undecided** — read it before making a
design call, the same way its counterpart in the SDK works. If a change
would contradict a stated goal or settle an open question, update
`GOALS.md` in the same change.

**Scaffold status.** Every resource that fits the simple CRUD template
(`resource_methods!`/`crud_methods!` in the SDK — list/get/create/update/
delete, no bulk/uploads/entity-actions) is wired up, 33 in total; `clients`
was the template the rest followed. See "Adding a resource" below for what
a resource needing more than the template (bulk actions, uploads, entity
actions, non-CRUD endpoints) will need beyond this.

## Development environment

Nix-managed, with its own `flake.nix` (not shared with the SDK's):

```sh
nix develop
```

The dev toolchain was originally pinned to match `invoiceninjars-sdk`'s own
(`devVersion` in each `flake.nix`), from back when this crate needed a sibling
checkout to build at all. That's no longer a hard requirement now that the
SDK is on crates.io, but the pin hasn't been revisited — if the two drift,
nothing here forces them back in sync, since the two flakes are independent
files.

**As of 0.2.0 (2026-10-25), the SDK is published on crates.io** and this
crate depends on it as a normal version requirement
(`invoiceninja-sdk = "0.2.0"` in `Cargo.toml`), not a path dependency. That
means:

- No `../invoiceninjars-sdk` checkout is needed to build this crate anymore.
  `cargo build` fetches `invoiceninja-sdk` from the registry like any other
  dependency.
- Picking up a change made in the SDK's working copy now requires that
  change to be published as a new crates.io release first by default — there
  is no more "edit the sibling checkout, rebuild here" loop out of the box.
  For active co-development across both crates, copy
  `.cargo/config.toml.example` to `.cargo/config.toml` (gitignored, so it
  stays per-developer and never affects CI or anyone else's build) to get
  a `paths` override back onto `../invoiceninjars-sdk`. It requires that
  checkout's `[package] version` to match this crate's `invoiceninja-sdk`
  requirement in `Cargo.toml` — bump both together if the SDK's working
  copy moves to an unreleased version. Delete `.cargo/config.toml` to go
  back to the published release.
- Pre-1.0, a y-bump (`0.2.x` → `0.3.0`) in the SDK is a breaking release per
  that crate's own `GOALS.md`. The default caret requirement already blocks
  picking that up automatically; bumping the version in `Cargo.toml` is a
  deliberate, reviewed step, not something `cargo update` should do on its
  own.

**Watch the SDK's own package/crate name.** It has changed at least once
already (`invoiceninjars` → `invoiceninja-sdk`, crate path `invoiceninja_sdk`)
without a corresponding change here landing first. If a build fails with "no
matching package named `invoiceninja-sdk` found" or an unresolved
`invoiceninja_sdk::` path, check the crates.io listing (or
`../invoiceninjars-sdk/Cargo.toml`'s `[package] name`, if you have a
checkout) first.

### Rust version and CI

| | Version | Declared in |
| --- | --- | --- |
| Dev toolchain | 1.99.1 | `devVersion` in `flake.nix`, `test` job in `.github/workflows/ci.yml` |
| MSRV | 1.88 | `rust-version` in `Cargo.toml` (inherited from the SDK's own floor — see the comment there), `msrv` job in `ci.yml` |

Unlike `invoiceninjars-sdk`'s `flake.nix`, this crate's has no separate
`msrvVersion`/toolchain — the `msrv` CI job is the only place 1.88 actually
gets built against, not a Nix devShell. If that ever earns its keep, mirror
the SDK's `msrvToolchain`/`shellFor` pattern rather than inventing a new one.

CI runs on GitHub Actions, against the GitHub push mirror (Gitea is
canonical — see `.gitea`/remote config — but Actions run on GitHub, the
same arrangement as `invoiceninjars-sdk`'s own CI). Three jobs, mirroring
the SDK's `ci.yml` job-for-job:

- **`test`** — the pinned dev toolchain; `cargo build --all-targets`,
  `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`.
- **`msrv`** — build and test only, on the declared 1.88 floor. No lint or
  format check: those vary by release, and pinning them to the oldest
  supported toolchain isn't useful.
- **`stable`** (advisory) — latest stable `rustc`, runs on push/PR and
  weekly via `schedule` (Mondays 06:00 UTC) so a new Rust release that
  breaks the build or fires a new clippy lint doesn't sit unnoticed for
  weeks between pushes. Deliberately **not** `continue-on-error` — a real
  red X, not a silent annotation — so if branch protection ever requires
  status checks, keep `stable` **out** of the required list; that setting,
  not anything in `ci.yml`, is what should stop it blocking merges. Expect
  it red occasionally through no fault of the change under test, and fix it
  when bumping `devVersion`.

The `test` and `msrv` jobs skip scheduled runs (`if: github.event_name !=
'schedule'`) — a pinned toolchain against a locked dependency graph can't
change on a timer, so the weekly run is for `stable` only.

### Testing against a real server

`flake.nix` has an `invoiceninja-nixos` input (Gitea, same host as this
repo's own remote) and re-exports two of its `nixosConfigurations` as
`invoiceninja-test-nginx`/`invoiceninja-test-caddy` — real NixOS VMs
running actual Invoice Ninja (PHP/Laravel + MariaDB + Redis + chromium for
PDF generation), not a mock. Reused as-is rather than redefined here, same
principle as this crate being a client of the SDK rather than a second
implementation of it — see that repo's own `CLAUDE.md` for what's actually
inside.

```sh
nix build --accept-flake-config '.#nixosConfigurations.invoiceninja-test-nginx.config.system.build.vm'
QEMU_NET_OPTS="hostfwd=tcp::8080-:80" ./result/bin/run-nixos-vm
```

`--accept-flake-config` is what lets the `nixConfig` cachix substituter
declared at the top of this file actually get used — without it (or if the
daemon's `trusted-users` rejects it anyway, which it did when scripted
through an agent sandbox in practice — built fine regardless, just slower)
Nix silently falls back to building from source. `QEMU_NET_OPTS` forwards
the guest's port 80 to host `8080`; it's a runtime flag to the generated
`run-nixos-vm` script, not baked into the VM config, so it has to be set on
every boot.

The VM has one fixed admin account baked in by the upstream repo's own test
fixtures (`admin@email.com` / `password`, from its `tests/invoice_ninja_test_password`) —
not meant to be secret, since it only ever exists inside a throwaway local
VM. There's no standing API token to copy anywhere; get one fresh each boot
via `/api/v1/login` (the SDK and CLI only ever use API-token auth, never
this login endpoint themselves — it's purely a test-fixture bootstrap
step):

```sh
TOKEN=$(curl -s -X POST http://localhost:8080/api/v1/login \
    -H 'Content-Type: application/json' \
    -d '{"email":"admin@email.com","password":"password"}' \
  | jq -r '.data[0].token.token')
IN_API_TOKEN="$TOKEN" IN_BASE_URL=http://localhost:8080/api/v1 \
  cargo run -- clients list
```

`jq` is in this flake's `devShells.default` for exactly this. There's also
an automated check (`checks.invoiceninja-cli` in `flake.nix`, backed by
`tests/`) that boots the same kind of VM and runs this CLI's own binary
against it non-interactively — see `tests/nixos-test.nix` for exactly what
it exercises. Verified working end to end (boot → login → token →
`clients create`/`list`) as of writing.

## Common commands

```sh
cargo build
cargo run -- clients list          # needs IN_API_TOKEN, or --token
cargo fmt --check
cargo clippy --all-targets
```

No test suite yet — nothing here has behavior worth testing beyond what
`clap`'s derive and the SDK's own test suite already cover. Add one (clap's
`Cli::try_parse_from` is the usual way to test argument parsing without
spawning a process) when a command does more than translate flags into an
SDK call.

## Architecture

- `src/cli.rs` — all `clap` derive types. `Cli` holds the global
  options (`--base-url`, `--token`, `--api-password`, `--output`) and the
  top-level `Command` subcommand enum. **`--token` can't be clap-`required`**
  because clap rejects `required` combined with `global` (global is what
  lets `--token` appear after the resource subcommand too, not just before
  it) — the missing case is caught in `config::build_client` instead, with a
  message naming both the flag and the env var. Don't try to make it
  `required` again; the comment on the field explains the panic you'd get.
- `src/config.rs` — `build_client` turns a parsed `Cli` into an
  `InvoiceNinjaClient`, including the manual token check above.
- `src/body.rs` — how `create`/`update` get their JSON body (`--data`
  inline, `--file` from disk or `-` for stdin, or stdin by default), plus
  `merge_shallow`, a one-level-deep object overlay.
- `src/output.rs` — `print_json`, shared by every resource's `--output
  json`. Table rendering is **not** here; it's per-resource (see below),
  since there's no generic notion of which columns matter.
- `src/commands/clients.rs` — the first resource implemented, and the
  template every other `src/commands/<resource>.rs` follows (33 of them now;
  see "Adding a resource" below).

### The `update` fetch-merge-write pattern

`update` is not a thin wrapper over the SDK's `update_client`. It:

1. Fetches the current record with `get_client`.
2. Serializes it to `serde_json::Value` and shallow-merges the caller's
   `--data`/`--file` JSON on top (`body::merge_shallow`).
3. Deserializes the merged value back into the model and calls
   `update_client`.

This exists because Invoice Ninja's `PUT` is a full replace — there's no
PATCH route — and every non-`Option` field on an SDK model decodes a missing
key as that type's default (`""`, `0`, `false`; see
`invoiceninja_sdk`'s `null_to_default`). Sending the caller's patch alone
would silently blank out every field they didn't mention. **Keep this
pattern for every resource's `update`** — it's the CLI-level version of the
SDK's own "never lose data in a round trip" goal, not a one-off for clients.

`create` has no such step: a partial JSON body decodes straight into the
model, since the same defaulting that makes a bare patch dangerous for
`update` is exactly what you want when creating a record whose fields you
haven't all thought about (the API does the equivalent server-side).

### Adding a resource

Each resource beyond `clients` should follow the same shape:

1. Add a variant to `Command` in `src/cli.rs` (e.g. `Invoices(ResourceArgs)`)
   — `ResourceArgs` and `Action` are already generic across resources, reuse
   them as-is.
2. Add `src/commands/<resource>.rs` mirroring `clients.rs`: a `run` dispatcher
   over `Action`, `list`/`get`/`create`/`update`/`delete` functions calling
   the matching `list_<resource>`/`get_<resource>`/… methods on
   `InvoiceNinjaClient`, and a `print_table` choosing that resource's own key
   columns.
3. Register the module in `src/commands/mod.rs` and the dispatch arm in
   `main.rs`'s `match cli.command`.
4. Check `../invoiceninjars-sdk/src/client.rs` for whether the resource has a
   `/bulk` route (most do; a few — `client_gateway_tokens`, `companies`,
   `system_logs` — don't, per that crate's `crud_methods!` vs
   `resource_methods!` split). This CLI doesn't expose bulk actions yet
   either way; that's a separate addition.

Resources with entity actions, document uploads, or non-CRUD endpoints
(reports, charts, search, statics, PDF downloads, …) will need more than
this template once they're added — the SDK's own `CLAUDE.md` describes which
resources have which extras.
