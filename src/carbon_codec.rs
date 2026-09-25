//! Canonical bounded Carbon JSON validation shared by the runtime outbox and
//! the optional Lightyear transport.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{Map, Value};

pub(crate) const MAX_CARBON_BINARY_BYTES: usize = 32 * 1024 * 1024;
pub(crate) const MAX_CARBON_TOTAL_BINARY_BYTES: usize = 48 * 1024 * 1024;
pub(crate) const MAX_CARBON_STRING_BYTES: usize = 32 * 1024 * 1024;
pub(crate) const MAX_CARBON_TOTAL_STRING_BYTES: usize = 48 * 1024 * 1024;
pub(crate) const MAX_CARBON_VALUE_DEPTH: usize = 64;
pub(crate) const MAX_CARBON_VALUE_NODES: usize = 200_000;

#[derive(Default)]
struct CanonicalBudget {
    nodes: usize,
    binary_bytes: usize,
    string_bytes: usize,
}

pub(crate) fn decode_canonical(value: Value) -> Result<Value, String> {
    decode_canonical_inner(value, 0, &mut CanonicalBudget::default())
}

fn decode_canonical_inner(
    value: Value,
    depth: usize,
    budget: &mut CanonicalBudget,
) -> Result<Value, String> {
    budget.nodes = budget
        .nodes
        .checked_add(1)
        .ok_or_else(|| "Carbon value node count overflowed".to_owned())?;
    if budget.nodes > MAX_CARBON_VALUE_NODES || depth > MAX_CARBON_VALUE_DEPTH {
        return Err("Carbon value exceeds the nesting/node limit".into());
    }
    match value {
        Value::Array(values) => values
            .into_iter()
            .map(|value| decode_canonical_inner(value, depth + 1, budget))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(mut object) => {
            let has_tuple = object.contains_key("$destiny_bevy_tuple_v1");
            let has_bytes = object.contains_key("$destiny_bevy_bytes_v1");
            if (has_tuple || has_bytes) && object.len() != 1 {
                return Err("Carbon object mixes a reserved codec key with data".into());
            }
            if has_tuple {
                let tuple = object
                    .remove("$destiny_bevy_tuple_v1")
                    .ok_or_else(|| "invalid Carbon tuple tag".to_owned())?;
                if !tuple.is_array() {
                    return Err("invalid Carbon tuple tag".into());
                }
                return decode_canonical_inner(tuple, depth + 1, budget);
            }
            if has_bytes {
                let encoded = object
                    .get("$destiny_bevy_bytes_v1")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "invalid Carbon binary tag".to_owned())?;
                if encoded.len() > MAX_CARBON_BINARY_BYTES.saturating_mul(4) / 3 + 16 {
                    return Err("Carbon binary tag exceeds the byte limit".into());
                }
                let decoded = BASE64
                    .decode(encoded.as_bytes())
                    .map_err(|_| "invalid Carbon binary tag".to_owned())?;
                if BASE64.encode(&decoded) != encoded {
                    return Err("invalid Carbon binary tag".into());
                }
                if decoded.len() > MAX_CARBON_BINARY_BYTES {
                    return Err("Carbon binary value exceeds the byte limit".into());
                }
                budget.binary_bytes = budget
                    .binary_bytes
                    .checked_add(decoded.len())
                    .ok_or_else(|| "Carbon binary byte count overflowed".to_owned())?;
                if budget.binary_bytes > MAX_CARBON_TOTAL_BINARY_BYTES {
                    return Err("Carbon binary values exceed the total byte limit".into());
                }
                return Ok(Value::Object(object));
            }
            let mut decoded = Map::new();
            for (key, value) in object {
                add_string_budget(&key, budget)?;
                decoded.insert(
                    key,
                    decode_canonical_inner(value, depth + 1, budget)?,
                );
            }
            Ok(Value::Object(decoded))
        }
        Value::String(value) => {
            add_string_budget(&value, budget)?;
            Ok(Value::String(value))
        }
        Value::Number(number) => {
            if number.is_i64()
                || number
                    .as_f64()
                    .is_some_and(|value| value.is_finite() && number.is_f64())
            {
                Ok(Value::Number(number))
            } else {
                Err("Carbon integer must fit a signed 64-bit value".into())
            }
        }
        other => Ok(other),
    }
}

fn add_string_budget(value: &str, budget: &mut CanonicalBudget) -> Result<(), String> {
    if value.len() > MAX_CARBON_STRING_BYTES {
        return Err("Carbon string exceeds the byte limit".into());
    }
    budget.string_bytes = budget
        .string_bytes
        .checked_add(value.len())
        .ok_or_else(|| "Carbon string byte count overflowed".to_owned())?;
    if budget.string_bytes > MAX_CARBON_TOTAL_STRING_BYTES {
        return Err("Carbon strings exceed the total byte limit".into());
    }
    Ok(())
}
