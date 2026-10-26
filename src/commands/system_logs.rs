//! `system-logs` subcommand. Same shape as `clients.rs` — see the "Adding a
//! resource" section in CLAUDE.md. Uses the SDK's `crud_methods!` rather
//! than `resource_methods!` (no `/bulk` route), which doesn't change
//! anything here — this CLI doesn't expose bulk actions yet either way.

use invoiceninja_sdk::models::{ListParams, SystemLog};
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

    let system_logs = if args.all {
        client.list_all_system_logs(&params).await?
    } else {
        client.list_system_logs(&params).await?
    };

    render(&system_logs, format)
}

async fn get(
    client: &InvoiceNinjaClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let found = client.get_system_log(id).await?;
    render(std::slice::from_ref(&found), format)
}

async fn create(
    client: &InvoiceNinjaClient,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = body::load(&body_args)?;
    let model: SystemLog = serde_json::from_value(value)?;
    let created = client.create_system_log(&model).await?;
    render(std::slice::from_ref(&created), format)
}

async fn update(
    client: &InvoiceNinjaClient,
    id: &str,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let patch = body::load(&body_args)?;
    let current = client.get_system_log(id).await?;
    let merged = body::merge_shallow(serde_json::to_value(&current)?, patch);
    let model: SystemLog = serde_json::from_value(merged)?;
    let updated = client.update_system_log(id, &model).await?;
    render(std::slice::from_ref(&updated), format)
}

async fn delete(client: &InvoiceNinjaClient, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    client.delete_system_log(id).await?;
    println!("deleted {id}");
    Ok(())
}

fn render(
    system_logs: &[SystemLog],
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => output::print_json(system_logs)?,
        OutputFormat::Table => print_table(system_logs),
    }
    Ok(())
}

fn print_table(system_logs: &[SystemLog]) {
    if system_logs.is_empty() {
        println!("(no records)");
        return;
    }
    println!(
        "{:<20}  {:<20}  {:>10}  {:>10}  {:>10}",
        "id", "client_id", "event_id", "category_id", "type_id"
    );
    for l in system_logs {
        println!(
            "{:<20}  {:<20}  {:>10}  {:>10}  {:>10}",
            l.id.as_deref().unwrap_or(""),
            truncate(&l.client_id, 20),
            l.event_id,
            l.category_id,
            l.type_id
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
