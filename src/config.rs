use invoiceninja_sdk::{InvoiceNinjaClient, HOSTED_BASE_URL};

use crate::cli::Cli;

/// Builds the SDK client from the parsed CLI args: the base URL (defaulting
/// to the hosted app), the token, and the optional account password for the
/// `password_protected` endpoints.
///
/// Checks for a missing token itself rather than leaning on clap, since
/// `token` can't be marked clap-`required` — see the comment on `Cli::token`.
pub fn build_client(cli: &Cli) -> Result<InvoiceNinjaClient, Box<dyn std::error::Error>> {
    let token = cli
        .token
        .clone()
        .ok_or("missing API token: pass --token or set IN_API_TOKEN")?;
    let base_url = cli
        .base_url
        .clone()
        .unwrap_or_else(|| HOSTED_BASE_URL.to_string());
    let client = InvoiceNinjaClient::new(base_url, token)?;
    match &cli.api_password {
        Some(password) => Ok(client.with_api_password(password.clone())?),
        None => Ok(client),
    }
}
