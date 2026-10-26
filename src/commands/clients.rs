//! `clients` subcommand. First resource wired up, and the template for
//! adding the rest — see the "Adding a resource" section in CLAUDE.md.

use invoiceninja_sdk::models::{Client, ListParams};
use invoiceninja_sdk::InvoiceNinjaClient;

use crate::body;
use crate::cli::{Action, BodyArgs, ListArgs, OutputFormat, ResourceArgs};
use crate::output;

pub async fn run(
    client: &InvoiceNinjaClient,
    args: ResourceArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match args.action {
        Action::List(list_args) => list(client, list_args, format).await,
        Action::Get { id } => get(client, &id, format).await,
        Action::Create(body_args) => create(client, body_args, format).await,
        Action::Update { id, body } => update(client, &id, body, format).await,
        Action::Delete { id } => delete(client, &id).await,
    }
}

async fn list(
    client: &InvoiceNinjaClient,
    args: ListArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut params = ListParams::new();
    if let Some(page) = args.page {
        params = params.page(page);
    }
    if let Some(per_page) = args.per_page {
        params = params.per_page(per_page);
    }
    if let Some(sort) = args.sort {
        params = params.sort(sort);
    }
    if let Some(filter) = args.filter {
        params = params.filter(filter);
    }
    if let Some(status) = args.status {
        params = params.status(status);
    }
    if let Some(include) = args.include {
        params = params.include(include);
    }
    for (k, v) in args.params {
        params = params.param(k, v);
    }

    let clients = if args.all {
        client.list_all_clients(&params).await?
    } else {
        client.list_clients(&params).await?
    };

    render(&clients, format)
}

async fn get(
    client: &InvoiceNinjaClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let found = client.get_client(id).await?;
    render(std::slice::from_ref(&found), format)
}

async fn create(
    client: &InvoiceNinjaClient,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = body::load(&body_args)?;
    // Missing fields in `value` decode fine: every non-Option field on
    // `Client` is `#[serde(default, deserialize_with = "null_to_default")]`,
    // so a partial object (e.g. just `{"name": "..."}`) fills the rest with
    // the model's defaults, same as the API does server-side on create.
    let model: Client = serde_json::from_value(value)?;
    let created = client.create_client(&model).await?;
    render(std::slice::from_ref(&created), format)
}

async fn update(
    client: &InvoiceNinjaClient,
    id: &str,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let patch = body::load(&body_args)?;
    let current = client.get_client(id).await?;
    let merged = body::merge_shallow(serde_json::to_value(&current)?, patch);
    let model: Client = serde_json::from_value(merged)?;
    let updated = client.update_client(id, &model).await?;
    render(std::slice::from_ref(&updated), format)
}

async fn delete(client: &InvoiceNinjaClient, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    client.delete_client(id).await?;
    println!("deleted {id}");
    Ok(())
}

fn render(clients: &[Client], format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => output::print_json(clients)?,
        OutputFormat::Table => print_table(clients),
    }
    Ok(())
}

/// The columns considered "key" for a client. Every resource module picks
/// its own this way — there's no generic notion of which fields matter
/// enough for a table view.
fn print_table(clients: &[Client]) {
    if clients.is_empty() {
        println!("(no records)");
        return;
    }
    println!(
        "{:<20}  {:<30}  {:>12}  {:>12}",
        "id", "name", "balance", "paid_to_date"
    );
    for c in clients {
        println!(
            "{:<20}  {:<30}  {:>12.2}  {:>12.2}",
            c.id.as_deref().unwrap_or(""),
            truncate(&c.name, 30),
            c.balance,
            c.paid_to_date
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
    }
}
