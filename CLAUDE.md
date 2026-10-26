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

**Early scaffold.** Six resources are wired up (`clients`, `invoices`,
`quotes`, `payments`, `products`, `expenses`) out of ~34; `clients` was the
template the other five followed. See "Adding a resource" below before
writing the next one.

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
- `src/commands/clients.rs` — the one resource implemented so far, and the
  template for the rest.

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
