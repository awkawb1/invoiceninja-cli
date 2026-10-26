# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Pre-1.0, per `GOALS.md`, a minor release can still break flags or output —
this file is where that gets written down instead of discovered at the
command line.

## [Unreleased]

### Added

- 33 resources, every one of them with a simple `list`/`get`/`create`/
  `update`/`delete` API in the SDK (`resource_methods!`/`crud_methods!`, no
  bulk/uploads/entity-actions) — see `invoiceninja-cli --help` for the
  full list. `list` takes `--all` for every page, `--status`, and
  `--param key=value` for anything `ListParams` doesn't have a typed
  setter for. `quotes` and `credits` are `invoices` under different
  endpoints — Invoice Ninja stores all three in the same table,
  distinguished by `entity_type`, so `Quote` and `Credit` are type aliases
  for `Invoice` in the SDK and `commands/quotes.rs`/`commands/credits.rs`
  are otherwise identical to `commands/invoices.rs`. Likewise
  `recurring_quotes`/`RecurringQuote` mirror `recurring_invoices`/
  `RecurringInvoice`.
- `update` fetches the current record, shallow-merges the caller's
  `--data`/`--file` JSON on top, and writes the merged result back, rather
  than sending the patch alone — Invoice Ninja's `PUT` is a full replace.
  See `CLAUDE.md` and the comment in `src/body.rs`.
- `--output json` on every command for the model's full `Serialize` output
  (including `extra`), suitable as input to a later `update`/`create`; the
  default table view is a convenience subset of columns.
- Global `--base-url`, `--token`, and `--api-password` flags, each with a
  matching environment variable (`IN_BASE_URL`, `IN_API_TOKEN`,
  `IN_API_PASSWORD`); both secrets are hidden from `--help` and any echoed
  default.
- Depends on [`invoiceninja-sdk`](https://crates.io/crates/invoiceninja-sdk)
  0.2.0 from crates.io.
- A `flake.nix` with `devShells.default`, `packages.default` (built with a
  pinned toolchain), and `overlays.default` (for pulling `invoiceninja-cli`
  into another flake's own `pkgs`).
- CI on GitHub Actions, mirroring `invoiceninja-sdk`'s own `ci.yml`: a
  pinned-toolchain `test` job, a build-only `msrv` job on the declared 1.88
  floor, and an advisory `stable` job.
