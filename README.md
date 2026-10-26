# invoiceninja-cli

A command-line client for the [Invoice Ninja](https://www.invoiceninja.com/)
v5 API, built on [`invoiceninja-sdk`](https://crates.io/crates/invoiceninja-sdk).

> **Early scaffold.** `clients`, `invoices`, `quotes`, `payments`, `products`,
> and `expenses` are wired up so far, out of ~34 resources — see `CLAUDE.md`
> for the pattern the rest follow. [`GOALS.md`](GOALS.md) covers what the
> crate is for and what it is not.

## Development

Enter the dev shell (Rust toolchain via `rust-overlay`, matching the SDK's):

```sh
nix develop
```

## Usage

Every subcommand needs an API token and, for a self-hosted install, a base
URL. Both can be passed as flags or environment variables (the same names
the SDK's own examples use):

```sh
export IN_API_TOKEN=...
# export IN_BASE_URL=https://your-domain.example/api/v1   # self-hosted only

invoiceninja-cli clients list
invoiceninja-cli clients list --all --status active,archived
invoiceninja-cli clients get <id>
invoiceninja-cli clients create --data '{"name": "Acme Inc"}'
invoiceninja-cli clients update <id> --data '{"city": "Springfield"}'
invoiceninja-cli clients delete <id>
```

`create` and `update` read the record body as JSON, from `--data` inline,
`--file <path>` (`-` for stdin), or stdin if neither flag is given.
`update` fetches the current record first and overlays the given fields on
top of it, so you only need to mention what's changing — see the comment in
`src/body.rs` for why that matters (Invoice Ninja's `PUT` is a full replace,
not a patch).

Add `--output json` to any command for the full record instead of the
table view's few key columns:

```sh
invoiceninja-cli --output json clients get <id>
```

Every other wired-up resource (`invoices`, `quotes`, `payments`, `products`,
`expenses`) follows the same `list`/`get`/`create`/`update`/`delete` shape,
e.g. `invoiceninja-cli invoices list --status active` or
`invoiceninja-cli payments get <id>`.
