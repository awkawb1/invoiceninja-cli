//! `bank-transaction-rules` subcommand. Same shape as `clients.rs` — see
//! the "Adding a resource" section in CLAUDE.md.

use invoiceninja_sdk::models::{BankTransactionRule, ListParams};
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

    let bank_transaction_rules = if args.all {
        client.list_all_bank_transaction_rules(&params).await?
    } else {
        client.list_bank_transaction_rules(&params).await?
    };

    render(&bank_transaction_rules, format)
}

async fn get(
    client: &InvoiceNinjaClient,
    id: &str,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let found = client.get_bank_transaction_rule(id).await?;
    render(std::slice::from_ref(&found), format)
}

async fn create(
    client: &InvoiceNinjaClient,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = body::load(&body_args)?;
    let model: BankTransactionRule = serde_json::from_value(value)?;
    let created = client.create_bank_transaction_rule(&model).await?;
    render(std::slice::from_ref(&created), format)
}

async fn update(
    client: &InvoiceNinjaClient,
    id: &str,
    body_args: BodyArgs,
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let patch = body::load(&body_args)?;
    let current = client.get_bank_transaction_rule(id).await?;
    let merged = body::merge_shallow(serde_json::to_value(&current)?, patch);
    let model: BankTransactionRule = serde_json::from_value(merged)?;
    let updated = client.update_bank_transaction_rule(id, &model).await?;
    render(std::slice::from_ref(&updated), format)
}

async fn delete(client: &InvoiceNinjaClient, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    client.delete_bank_transaction_rule(id).await?;
    println!("deleted {id}");
    Ok(())
}

fn render(
    bank_transaction_rules: &[BankTransactionRule],
    format: OutputFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => output::print_json(bank_transaction_rules)?,
        OutputFormat::Table => print_table(bank_transaction_rules),
    }
    Ok(())
}

fn print_table(bank_transaction_rules: &[BankTransactionRule]) {
    if bank_transaction_rules.is_empty() {
        println!("(no records)");
        return;
    }
    println!(
        "{:<20}  {:<30}  {:<15}  {:<8}",
        "id", "name", "applies_to", "auto"
    );
    for r in bank_transaction_rules {
        println!(
            "{:<20}  {:<30}  {:<15}  {:<8}",
            r.id.as_deref().unwrap_or(""),
            truncate(&r.name, 30),
            truncate(&r.applies_to, 15),
            r.auto_convert
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
