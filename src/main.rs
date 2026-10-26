mod body;
mod cli;
mod commands;
mod config;
mod output;

use clap::Parser;

use cli::{Cli, Command};

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let client = config::build_client(&cli)?;

    match cli.command {
        Command::Clients(args) => commands::clients::run(&client, args, cli.output).await,
        Command::Invoices(args) => commands::invoices::run(&client, args, cli.output).await,
        Command::Quotes(args) => commands::quotes::run(&client, args, cli.output).await,
        Command::Credits(args) => commands::credits::run(&client, args, cli.output).await,
        Command::Payments(args) => commands::payments::run(&client, args, cli.output).await,
        Command::Products(args) => commands::products::run(&client, args, cli.output).await,
        Command::Expenses(args) => commands::expenses::run(&client, args, cli.output).await,
        Command::Projects(args) => commands::projects::run(&client, args, cli.output).await,
        Command::Tasks(args) => commands::tasks::run(&client, args, cli.output).await,
        Command::Vendors(args) => commands::vendors::run(&client, args, cli.output).await,
        Command::PurchaseOrders(args) => {
            commands::purchase_orders::run(&client, args, cli.output).await
        }
        Command::RecurringInvoices(args) => {
            commands::recurring_invoices::run(&client, args, cli.output).await
        }
        Command::RecurringQuotes(args) => {
            commands::recurring_quotes::run(&client, args, cli.output).await
        }
        Command::RecurringExpenses(args) => {
            commands::recurring_expenses::run(&client, args, cli.output).await
        }
        Command::TaxRates(args) => commands::tax_rates::run(&client, args, cli.output).await,
        Command::ExpenseCategories(args) => {
            commands::expense_categories::run(&client, args, cli.output).await
        }
        Command::TaskStatuses(args) => {
            commands::task_statuses::run(&client, args, cli.output).await
        }
        Command::PaymentTerms(args) => {
            commands::payment_terms::run(&client, args, cli.output).await
        }
        Command::Subscriptions(args) => {
            commands::subscriptions::run(&client, args, cli.output).await
        }
        Command::BankTransactions(args) => {
            commands::bank_transactions::run(&client, args, cli.output).await
        }
        Command::Webhooks(args) => commands::webhooks::run(&client, args, cli.output).await,
        Command::BankIntegrations(args) => {
            commands::bank_integrations::run(&client, args, cli.output).await
        }
        Command::BankTransactionRules(args) => {
            commands::bank_transaction_rules::run(&client, args, cli.output).await
        }
        Command::CompanyGateways(args) => {
            commands::company_gateways::run(&client, args, cli.output).await
        }
        Command::Designs(args) => commands::designs::run(&client, args, cli.output).await,
        Command::GroupSettings(args) => {
            commands::group_settings::run(&client, args, cli.output).await
        }
        Command::Locations(args) => commands::locations::run(&client, args, cli.output).await,
        Command::Tags(args) => commands::tags::run(&client, args, cli.output).await,
        Command::TaskSchedulers(args) => {
            commands::task_schedulers::run(&client, args, cli.output).await
        }
        Command::Tokens(args) => commands::tokens::run(&client, args, cli.output).await,
        Command::ClientGatewayTokens(args) => {
            commands::client_gateway_tokens::run(&client, args, cli.output).await
        }
        Command::Companies(args) => commands::companies::run(&client, args, cli.output).await,
        Command::SystemLogs(args) => commands::system_logs::run(&client, args, cli.output).await,
    }
}
