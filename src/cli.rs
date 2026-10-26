use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// A command-line client for the Invoice Ninja v5 API, built on the
/// `invoiceninja-sdk` crate.
#[derive(Parser)]
#[command(name = "invoiceninja-cli", version, about)]
pub struct Cli {
    /// API root, including the `/api/v1` suffix. Defaults to the hosted app
    /// (`invoicing.co`); a self-hosted install must pass this.
    #[arg(long, env = "IN_BASE_URL", global = true)]
    pub base_url: Option<String>,

    /// API token. Required for every call.
    ///
    /// Typed `Option` rather than clap-`required`: clap refuses to combine
    /// `required` with `global` (needed so this can also be passed after the
    /// subcommand), so the missing case is checked in `config::build_client`
    /// instead, with a message that names both ways to supply it.
    /// `hide_env_values` keeps it out of `--help`, which otherwise echoes
    /// env-sourced defaults back verbatim.
    #[arg(long, env = "IN_API_TOKEN", global = true, hide_env_values = true)]
    pub token: Option<String>,

    /// Account password, needed only for the endpoints Invoice Ninja marks
    /// `password_protected` (reading or writing a user, purging a client).
    #[arg(long, env = "IN_API_PASSWORD", global = true, hide_env_values = true)]
    pub api_password: Option<String>,

    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputFormat {
    /// A few key columns, one record per line. Which columns count as "key"
    /// is a per-resource choice, made in that resource's command module.
    Table,
    /// The full record(s), pretty-printed. Always lossless — use this to see
    /// every field a table view leaves out, including `extra`.
    Json,
}

/// One variant per resource. Every resource that fits the simple CRUD
/// template (`resource_methods!`/`crud_methods!` in the SDK) is wired up;
/// see CLAUDE.md's "Adding a resource" for what's deliberately left out
/// (bulk actions, uploads, entity actions, non-CRUD endpoints).
#[derive(Subcommand)]
pub enum Command {
    /// Manage clients
    Clients(ResourceArgs),
    /// Manage invoices
    Invoices(ResourceArgs),
    /// Manage quotes
    Quotes(ResourceArgs),
    /// Manage credits
    Credits(ResourceArgs),
    /// Manage payments
    Payments(ResourceArgs),
    /// Manage products
    Products(ResourceArgs),
    /// Manage expenses
    Expenses(ResourceArgs),
    /// Manage projects
    Projects(ResourceArgs),
    /// Manage tasks
    Tasks(ResourceArgs),
    /// Manage vendors
    Vendors(ResourceArgs),
    /// Manage purchase orders
    PurchaseOrders(ResourceArgs),
    /// Manage recurring invoices
    RecurringInvoices(ResourceArgs),
    /// Manage recurring quotes
    RecurringQuotes(ResourceArgs),
    /// Manage recurring expenses
    RecurringExpenses(ResourceArgs),
    /// Manage tax rates
    TaxRates(ResourceArgs),
    /// Manage expense categories
    ExpenseCategories(ResourceArgs),
    /// Manage task statuses
    TaskStatuses(ResourceArgs),
    /// Manage payment terms
    PaymentTerms(ResourceArgs),
    /// Manage subscriptions
    Subscriptions(ResourceArgs),
    /// Manage bank transactions
    BankTransactions(ResourceArgs),
    /// Manage webhooks
    Webhooks(ResourceArgs),
    /// Manage bank integrations
    BankIntegrations(ResourceArgs),
    /// Manage bank transaction rules
    BankTransactionRules(ResourceArgs),
    /// Manage company gateways
    CompanyGateways(ResourceArgs),
    /// Manage designs
    Designs(ResourceArgs),
    /// Manage group settings
    GroupSettings(ResourceArgs),
    /// Manage locations
    Locations(ResourceArgs),
    /// Manage tags
    Tags(ResourceArgs),
    /// Manage task schedulers
    TaskSchedulers(ResourceArgs),
    /// Manage API tokens
    Tokens(ResourceArgs),
    /// Manage client gateway tokens
    ClientGatewayTokens(ResourceArgs),
    /// Manage companies
    Companies(ResourceArgs),
    /// Manage system logs
    SystemLogs(ResourceArgs),
}

#[derive(Args)]
pub struct ResourceArgs {
    #[command(subcommand)]
    pub action: Action,
}

/// The five actions every resource exposes. `Create`/`Update` share
/// `BodyArgs` because both take a JSON record; see `body::load`.
#[derive(Subcommand)]
pub enum Action {
    /// List records (one page by default; pass --all to follow pagination)
    List(ListArgs),
    /// Fetch one record by id
    Get { id: String },
    /// Create a record from a JSON body
    Create(BodyArgs),
    /// Update a record from a JSON body, keeping fields you don't mention.
    ///
    /// Fetches the record, overlays the given JSON on top, and writes the
    /// merged result back — see the comment in `commands::clients::update`
    /// for why a plain PUT of just the patch would blank out the rest.
    Update {
        id: String,
        #[command(flatten)]
        body: BodyArgs,
    },
    /// Delete (soft-delete) a record by id
    Delete { id: String },
}

/// Mirrors the parameters every `ListParams` accepts in the SDK — see
/// `invoiceninja_sdk::models::ListParams`.
#[derive(Args)]
pub struct ListArgs {
    #[arg(long)]
    pub page: Option<u64>,
    #[arg(long)]
    pub per_page: Option<u64>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub filter: Option<String>,
    /// e.g. "active,archived,deleted" to include archived/deleted records
    #[arg(long)]
    pub status: Option<String>,
    /// Comma-separated relationships to eager-load, e.g. "contacts"
    #[arg(long)]
    pub include: Option<String>,
    /// Extra query parameter, as key=value. Repeatable, for the per-resource
    /// filters `ListParams::param` covers (`client_id`, `client_status`, …).
    #[arg(long = "param", value_parser = parse_key_val)]
    pub params: Vec<(String, String)>,
    /// Follow pagination and return every matching record instead of one
    /// page (`list_all_*` in the SDK rather than `list_*`).
    #[arg(long)]
    pub all: bool,
}

#[derive(Args)]
pub struct BodyArgs {
    /// The record body, as a JSON object, given inline.
    #[arg(long, conflicts_with = "file")]
    pub data: Option<String>,
    /// The record body, as a JSON object, read from a file ("-" for stdin).
    #[arg(long, conflicts_with = "data")]
    pub file: Option<PathBuf>,
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| format!("expected key=value, got `{s}`"))?;
    Ok((k.to_string(), v.to_string()))
}
