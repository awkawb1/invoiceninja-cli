# Goals

What this crate is trying to be, what it is deliberately not, and what is
still open. Same purpose as `invoiceninjars-sdk`'s own `GOALS.md`: several
choices here look like oversights until you know the reasoning, and that
reasoning should live here rather than only in commit messages.

## What this is for

A command-line way to drive the Invoice Ninja v5 API through
`invoiceninja-sdk`, for the cases Rust code is the wrong tool for: a quick
"what does this endpoint actually return", a one-off fix from a terminal, or
a shell script that doesn't want to be a Rust program. The SDK's own
`GOALS.md` names this exact need — its `examples/` cover ad-hoc probing
today, and this crate is the standalone version of that.

It is a client over the SDK, not a second implementation of anything the SDK
already does. If a command needs logic beyond "turn flags and JSON into one
SDK call", that is usually a sign the logic belongs in the SDK instead.

## Goals

### 1. Every command is a thin translation, with one documented exception

`list`, `get`, `create`, and `delete` do nothing but turn CLI flags into a
`ListParams`/model/id and call the matching SDK method. **`update` is the
one exception**, and it stays an exception rather than a pattern to extend
casually: it fetches the record, overlays the caller's JSON on top, and
writes the merged result back, because Invoice Ninja's `PUT` is a full
replace and the SDK's models default an omitted field rather than leaving it
alone. See the comment in `src/commands/clients.rs` and the matching section
in `CLAUDE.md`. Any other deviation from "thin translation" should be
written down the same way.

### 2. `--output json` is always the full, lossless record

The table view is a convenience subset — whatever columns a resource's
`print_table` picks — and it is allowed to leave things out. `--output json`
is not: it is always the model's own `Serialize` output, including
`extra`, so it is a drop-in input to `--data`/`--file` for a later `update`
or `create` and a reliable thing to pipe into `jq`. This is the CLI-level
version of the SDK's "never lose data in a round trip" goal.

### 3. One pattern across resources

Adding a resource should be mechanical, not a new design each time: a
`Command` variant, a `src/commands/<resource>.rs` with the same
`run`/`list`/`get`/`create`/`update`/`delete`/`print_table` shape as
`clients.rs`, and a registration line in two places. `CLAUDE.md`'s "Adding a
resource" section is the checklist; if a resource needs something the
template doesn't cover (bulk, uploads, entity actions, non-CRUD endpoints),
that is scope for a deliberate addition to the template, not a one-off.

### 4. Scriptable by default

Errors go to stderr and a failure exits non-zero; nothing is printed to
stdout on an error path that a script could mistake for a result. Secrets
(`--token`, `--api-password`) never appear in `--help` or in any echoed
default — see `hide_env_values` on each in `src/cli.rs`. The SDK already
keeps both out of its own `Debug` output; this crate's job is not to undo
that by printing them itself.

### 5. Inherit the SDK's safety defaults rather than re-deciding them

This crate does not expose its own retry/timeout policy or add a write path
the SDK doesn't already consider safe. If the SDK decides `POST` isn't
replayed on a 5xx because it might double-create an invoice, that reasoning
applies here unchanged — the CLI is not a place to relax it for convenience.

### 6. CI mirrors the SDK's, job for job

`.github/workflows/ci.yml` has the same three jobs as `invoiceninjars-sdk`'s
own — pinned-toolchain `test`, build-only `msrv` on the declared floor, and
an advisory `stable` that reports red but stays out of required status
checks — rather than inventing a different shape for this crate. See
`CLAUDE.md`'s "Rust version and CI" for the per-job reasoning; a deviation
from this shape should be written down there the same way `update`'s
exception is written down in goal 1.

## Non-goals

- **No Cargo workspace with the SDK.** Explained in `CLAUDE.md`: the SDK is
  its own git repo with its own CI and crates.io release, mirrored read-only
  to GitHub. Now that the SDK is published (0.2.0, 2026-10-25), the
  dependency is a plain crates.io version requirement — there isn't even a
  path dependency linking the two build graphs anymore, let alone a
  workspace. This doesn't mean co-development across both crates is gone:
  `.cargo/config.toml.example` documents a per-developer `paths` override
  back onto a local `../invoiceninjars-sdk` checkout, gitignored so it never
  becomes the default for anyone else or for CI. See `CLAUDE.md`.
- **No interactive mode or TUI.** Every command should work non-interactively
  from a script; a prompt-driven mode would be a different tool.
- **No local state, cache, or config file — yet.** Credentials and the base
  URL come from flags or environment variables only, resolved fresh on every
  invocation. See the open question below on multi-account profiles before
  assuming this is permanent.
- **No stability promise before 1.0.** Same posture as the SDK: flags are a
  compatibility surface too, and pre-1.0 they can change. Record breaking
  changes somewhere legible (a `CHANGELOG.md`, once there's enough history to
  need one) rather than letting them be discovered at the command line.

## Open questions

These are genuinely undecided; deciding one is a change to this file.

- **Bulk actions.** The SDK exposes `bulk_<resource>` (archive/restore/delete
  plus resource-specific actions) on every resource with a `/bulk` route.
  Nothing here calls it yet. Worth having, but a bulk `delete` from a CLI
  invites a worse mistake than a one-record `delete` — whether that needs a
  confirmation prompt, a `--yes` flag, or a dry-run preview is unresolved.
- **`--dry-run` for `create`/`update`/`delete`.** Would print what the merged
  body or the target of a delete would be without sending the request.
  Cheap for `create`/`delete`; for `update` it would mean exposing the
  fetch-merge result for inspection before the write half runs.
- **Multi-account profiles.** Flags and env vars cover one account per
  invocation. A config file keyed by profile name (base URL + token) would
  help anyone working against more than one instance regularly, at the cost
  of being the first place this crate stores anything on disk.
- **How closely should list flags track `ListParams`?** Right now `clients
  list` exposes exactly what `ListParams` has typed setters for, plus
  `--param key=value` for the rest. A per-resource convenience filter (e.g.
  a `--unpaid` shorthand for `client_status=unpaid`) would read nicer but
  means hand-picking which filters earn a dedicated flag per resource.
- **Whether the Nix packaging goes further than this repo's own flake.**
  `flake.nix` now has `packages.default`/`devShells.default` (built with a
  pinned `rust-overlay` toolchain, for parity between `nix build` and
  `nix develop`) and `overlays.default` (for another flake to pull
  `invoiceninja-cli` into its own `pkgs`, built with *that* nixpkgs's own
  `rustPlatform` instead of the pin). The overlay path means this crate's
  `rust-version = "1.88"` has to be met by whatever rustc the consuming
  channel ships — there's no guard here if it isn't, just a build failure
  over there. Whether that's worth submitting to nixpkgs proper (and how an
  MSRV bump would then need to be communicated beyond this repo) is
  unresolved.
