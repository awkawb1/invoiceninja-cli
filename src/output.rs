use serde::Serialize;

/// Prints a value as pretty JSON. Used for every resource's `--output json`
/// — the table view is per-resource (see `commands::clients::print_table`),
/// but JSON formatting isn't, since it's just the model's own `Serialize`.
pub fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
