//! `client-gateway-tokens` subcommand. Same shape as `clients.rs` — see
//! the "Adding a resource" section in CLAUDE.md. Uses the SDK's
//! `crud_methods!` rather than `resource_methods!` (no `/bulk` route), which
//! doesn't change anything here — this CLI doesn't expose bulk actions yet
//! either way.

use invoiceninja_sdk::models::{ClientGatewayToken, ListParams};
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

    let client_gateway_tokens = if args.all {
        client.list_all_client_gateway_tokens(&params).await?
    } else {
        client.list_client_gateway_tokens(&params).await?
    };

    render(&client_gateway_tokens, format)
}

async fn get(
    client: &InvoiceNinjaClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let found = client.get_client_gateway_token(id).await?;
    render(std::slice::from_ref(&found), format)
}

async fn create(
    client: &InvoiceNinjaClient,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = body::load(&body_args)?;
    let model: ClientGatewayToken = serde_json::from_value(value)?;
    let created = client.create_client_gateway_token(&model).await?;
    render(std::slice::from_ref(&created), format)
}

async fn update(
    client: &InvoiceNinjaClient,
    id: &str,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let patch = body::load(&body_args)?;
    let current = client.get_client_gateway_token(id).await?;
    let merged = body::merge_shallow(serde_json::to_value(&current)?, patch);
    let model: ClientGatewayToken = serde_json::from_value(merged)?;
    let updated = client.update_client_gateway_token(id, &model).await?;
    render(std::slice::from_ref(&updated), format)
}

async fn delete(client: &InvoiceNinjaClient, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    client.delete_client_gateway_token(id).await?;
    println!("deleted {id}");
    Ok(())
}

fn render(
    client_gateway_tokens: &[ClientGatewayToken],
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => output::print_json(client_gateway_tokens)?,
        OutputFormat::Table => print_table(client_gateway_tokens),
    }
    Ok(())
}

fn print_table(client_gateway_tokens: &[ClientGatewayToken]) {
    if client_gateway_tokens.is_empty() {
        println!("(no records)");
        return;
    }
    println!(
        "{:<20}  {:<20}  {:<20}  {:<10}",
        "id", "client_id", "company_gateway_id", "is_default"
    );
    for t in client_gateway_tokens {
        println!(
            "{:<20}  {:<20}  {:<20}  {:<10}",
            t.id.as_deref().unwrap_or(""),
            truncate(&t.client_id, 20),
            truncate(&t.company_gateway_id, 20),
            t.is_default
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
