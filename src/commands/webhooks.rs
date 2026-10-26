//! `webhooks` subcommand. Same shape as `clients.rs` — see the "Adding a
//! resource" section in CLAUDE.md.

use invoiceninja_sdk::models::{ListParams, Webhook};
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

    let webhooks = if args.all {
        client.list_all_webhooks(&params).await?
    } else {
        client.list_webhooks(&params).await?
    };

    render(&webhooks, format)
}

async fn get(
    client: &InvoiceNinjaClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let found = client.get_webhook(id).await?;
    render(std::slice::from_ref(&found), format)
}

async fn create(
    client: &InvoiceNinjaClient,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = body::load(&body_args)?;
    let model: Webhook = serde_json::from_value(value)?;
    let created = client.create_webhook(&model).await?;
    render(std::slice::from_ref(&created), format)
}

async fn update(
    client: &InvoiceNinjaClient,
    id: &str,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let patch = body::load(&body_args)?;
    let current = client.get_webhook(id).await?;
    let merged = body::merge_shallow(serde_json::to_value(&current)?, patch);
    let model: Webhook = serde_json::from_value(merged)?;
    let updated = client.update_webhook(id, &model).await?;
    render(std::slice::from_ref(&updated), format)
}

async fn delete(client: &InvoiceNinjaClient, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    client.delete_webhook(id).await?;
    println!("deleted {id}");
    Ok(())
}

fn render(webhooks: &[Webhook], format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => output::print_json(webhooks)?,
        OutputFormat::Table => print_table(webhooks),
    }
    Ok(())
}

fn print_table(webhooks: &[Webhook]) {
    if webhooks.is_empty() {
        println!("(no records)");
        return;
    }
    println!(
        "{:<20}  {:<10}  {:<40}  {:<8}",
        "id", "event_id", "target_url", "format"
    );
    for w in webhooks {
        println!(
            "{:<20}  {:<10}  {:<40}  {:<8}",
            w.id.as_deref().unwrap_or(""),
            w.event_id,
            truncate(&w.target_url, 40),
            w.format
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
