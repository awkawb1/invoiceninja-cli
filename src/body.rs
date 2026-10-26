use std::fs;
use std::io::Read;

use serde_json::Value;

use crate::cli::BodyArgs;

/// Reads the JSON body a `create`/`update` command was given: `--data`
/// inline, `--file` from disk (`-` for stdin), or stdin if neither flag was
/// passed. Errors if the result isn't a JSON object — every create/update
/// endpoint here takes a record, not a bare array or scalar.
pub fn load(body: &BodyArgs) -> Result<Value, Box<dyn std::error::Error>> {
    let text = match (&body.data, &body.file) {
        (Some(data), None) => data.clone(),
        (None, Some(path)) if path.as_os_str() == "-" => read_stdin()?,
        (None, Some(path)) => fs::read_to_string(path)?,
        (None, None) => read_stdin()?,
        // clap's `conflicts_with` on both fields rules this out before we get here.
        (Some(_), Some(_)) => unreachable!("--data and --file are mutually exclusive"),
    };

    let value: Value = serde_json::from_str(&text)?;
    if !value.is_object() {
        return Err(format!("expected a JSON object, got {value}").into());
    }
    Ok(value)
}

fn read_stdin() -> Result<String, Box<dyn std::error::Error>> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

/// Overlays `patch`'s keys onto `base`, one level deep.
///
/// `update` uses this so a caller can send just the fields they're changing
/// instead of retyping the whole record. That matters because Invoice
/// Ninja's `PUT` is a full replace — there's no PATCH route — and the SDK's
/// models default any field the caller omits to `""` / `0` / `false` rather
/// than leaving it alone. Sending the patch by itself would blank out every
/// field it didn't mention; merging onto a freshly-fetched copy keeps them.
pub fn merge_shallow(mut base: Value, patch: Value) -> Value {
    if let (Some(base_map), Value::Object(patch_map)) = (base.as_object_mut(), patch) {
        for (k, v) in patch_map {
            base_map.insert(k, v);
        }
    }
    base
}
