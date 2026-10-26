//! `invoices` subcommand. Same shape as `clients.rs` — see the "Adding a
//! resource" section in CLAUDE.md.

use std::io::Write;
use std::path::PathBuf;

use invoiceninja_sdk::models::{Invoice, ListParams};
use invoiceninja_sdk::InvoiceNinjaClient;

use crate::body;
use crate::cli::{BodyArgs, EntityAction, EntityResourceArgs, ListArgs, OutputFormat};
use crate::output;

pub async fn run(
    client: &InvoiceNinjaClient,
    args: EntityResourceArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match args.action {
        EntityAction::List(list_args) => list(client, list_args, format).await,
        EntityAction::Get { id } => get(client, &id, format).await,
        EntityAction::Create(body_args) => create(client, body_args, format).await,
        EntityAction::Update { id, body } => update(client, &id, body, format).await,
        EntityAction::Delete { id } => delete(client, &id).await,
        EntityAction::Action { id, action } => do_action(client, &id, &action, format).await,
        EntityAction::Download { ids, out } => download(client, &ids, out).await,
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

    let invoices = if args.all {
        client.list_all_invoices(&params).await?
    } else {
        client.list_invoices(&params).await?
    };

    render(&invoices, format)
}

async fn get(
    client: &InvoiceNinjaClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let found = client.get_invoice(id).await?;
    render(std::slice::from_ref(&found), format)
}

async fn create(
    client: &InvoiceNinjaClient,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = body::load(&body_args)?;
    let model: Invoice = serde_json::from_value(value)?;
    let created = client.create_invoice(&model).await?;
    render(std::slice::from_ref(&created), format)
}

async fn update(
    client: &InvoiceNinjaClient,
    id: &str,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let patch = body::load(&body_args)?;
    let current = client.get_invoice(id).await?;
    let merged = body::merge_shallow(serde_json::to_value(&current)?, patch);
    let model: Invoice = serde_json::from_value(merged)?;
    let updated = client.update_invoice(id, &model).await?;
    render(std::slice::from_ref(&updated), format)
}

async fn delete(client: &InvoiceNinjaClient, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    client.delete_invoice(id).await?;
    println!("deleted {id}");
    Ok(())
}

async fn do_action(
    client: &InvoiceNinjaClient,
    id: &str,
    action: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let updated = client.invoice_action(id, action).await?;
    render(std::slice::from_ref(&updated), format)
}

async fn download(
    client: &InvoiceNinjaClient,
    ids: &[String],
    out: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = if ids.len() == 1 {
        client.download_invoice(&ids[0]).await?
    } else {
        client.download_invoices(ids).await?
    };
    match out {
        Some(path) => std::fs::write(path, bytes)?,
        None => std::io::stdout().write_all(&bytes)?,
    }
    Ok(())
}

fn render(invoices: &[Invoice], format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => output::print_json(invoices)?,
        OutputFormat::Table => print_table(invoices),
    }
    Ok(())
}

fn print_table(invoices: &[Invoice]) {
    if invoices.is_empty() {
        println!("(no records)");
        return;
    }
    println!(
        "{:<20}  {:<15}  {:<20}  {:<8}  {:>12}  {:>12}",
        "id", "number", "client_id", "status", "amount", "balance"
    );
    for i in invoices {
        println!(
            "{:<20}  {:<15}  {:<20}  {:<8}  {:>12.2}  {:>12.2}",
            i.id.as_deref().unwrap_or(""),
            truncate(&i.number, 15),
            truncate(&i.client_id, 20),
            i.status_id,
            i.amount,
            i.balance
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
