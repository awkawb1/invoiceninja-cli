//! `tasks` subcommand. Same shape as `clients.rs` — see the "Adding a
//! resource" section in CLAUDE.md.

use invoiceninja_sdk::models::{ListParams, Task};
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

    let tasks = if args.all {
        client.list_all_tasks(&params).await?
    } else {
        client.list_tasks(&params).await?
    };

    render(&tasks, format)
}

async fn get(
    client: &InvoiceNinjaClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let found = client.get_task(id).await?;
    render(std::slice::from_ref(&found), format)
}

async fn create(
    client: &InvoiceNinjaClient,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = body::load(&body_args)?;
    let model: Task = serde_json::from_value(value)?;
    let created = client.create_task(&model).await?;
    render(std::slice::from_ref(&created), format)
}

async fn update(
    client: &InvoiceNinjaClient,
    id: &str,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let patch = body::load(&body_args)?;
    let current = client.get_task(id).await?;
    let merged = body::merge_shallow(serde_json::to_value(&current)?, patch);
    let model: Task = serde_json::from_value(merged)?;
    let updated = client.update_task(id, &model).await?;
    render(std::slice::from_ref(&updated), format)
}

async fn delete(client: &InvoiceNinjaClient, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    client.delete_task(id).await?;
    println!("deleted {id}");
    Ok(())
}

fn render(tasks: &[Task], format: OutputFormat) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => output::print_json(tasks)?,
        OutputFormat::Table => print_table(tasks),
    }
    Ok(())
}

fn print_table(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("(no records)");
        return;
    }
    println!(
        "{:<20}  {:<30}  {:<20}  {:<20}  {:<8}",
        "id", "description", "client_id", "project_id", "status"
    );
    for t in tasks {
        println!(
            "{:<20}  {:<30}  {:<20}  {:<20}  {:<8}",
            t.id.as_deref().unwrap_or(""),
            truncate(&t.description, 30),
            truncate(&t.client_id, 20),
            truncate(&t.project_id, 20),
            t.status_id
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
