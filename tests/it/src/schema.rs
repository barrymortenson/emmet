// SPDX-License-Identifier: AGPL-3.0-or-later
//! A small JSON Schema (2020-12) validator for the subset schemars writes: type, properties, required,
//! additionalProperties, items, minItems, maxItems, enum, const, anyOf, oneOf, allOf, and local `$ref`s. Enough to
//! check that every MCP tool's structured output matches the output schema it advertises (plan section 09).

use serde_json::Value;

/// Every place `value` breaks `schema`, as `path: problem`. Empty means it validates.
pub fn validate(schema: &Value, value: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    check(schema, schema, value, "$", &mut errors);
    errors
}

fn check(root: &Value, schema: &Value, value: &Value, path: &str, errors: &mut Vec<String>) {
    let s = match schema {
        Value::Bool(true) => return,
        Value::Bool(false) => {
            errors.push(format!("{path}: no value is allowed here"));
            return;
        }
        Value::Object(s) => s,
        _ => {
            errors.push(format!("{path}: the schema isn't an object"));
            return;
        }
    };
    if let Some(r) = s.get("$ref").and_then(Value::as_str) {
        match r
            .strip_prefix("#/")
            .map(|p| p.split('/').fold(Some(root), |v, k| v.and_then(|v| v.get(k))))
        {
            Some(Some(target)) => check(root, target, value, path, errors),
            _ => errors.push(format!("{path}: unresolved $ref {r}")),
        }
    }
    if let Some(t) = s.get("type") {
        let types: Vec<&str> = match t {
            Value::String(t) => vec![t.as_str()],
            Value::Array(ts) => ts.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        };
        if !types.iter().any(|t| is_type(value, t)) {
            errors.push(format!("{path}: expected {}, got {}", types.join(" or "), kind(value)));
            return;
        }
    }
    if let Some(Value::Array(options)) = s.get("enum")
        && !options.contains(value)
    {
        errors.push(format!(
            "{path}: {value} isn't one of {}",
            Value::Array(options.clone())
        ));
    }
    if let Some(c) = s.get("const")
        && c != value
    {
        errors.push(format!("{path}: expected {c}"));
    }
    for key in ["anyOf", "oneOf"] {
        if let Some(Value::Array(options)) = s.get(key) {
            let passing = options.iter().filter(|o| validate_at(root, o, value)).count();
            let ok = if key == "anyOf" { passing >= 1 } else { passing == 1 };
            if !ok {
                errors.push(format!("{path}: matches {passing} of the {key} options"));
            }
        }
    }
    if let Some(Value::Array(all)) = s.get("allOf") {
        for o in all {
            check(root, o, value, path, errors);
        }
    }
    if let Value::Object(obj) = value {
        let props = s.get("properties").and_then(Value::as_object);
        if let Some(Value::Array(required)) = s.get("required") {
            for r in required.iter().filter_map(Value::as_str) {
                if !obj.contains_key(r) {
                    errors.push(format!("{path}: missing required '{r}'"));
                }
            }
        }
        for (k, v) in obj {
            let sub = format!("{path}.{k}");
            match props.and_then(|p| p.get(k)) {
                Some(ps) => check(root, ps, v, &sub, errors),
                None => match s.get("additionalProperties") {
                    Some(Value::Bool(false)) => errors.push(format!("{sub}: not allowed")),
                    Some(extra) => check(root, extra, v, &sub, errors),
                    None => {}
                },
            }
        }
    }
    if let Value::Array(items) = value {
        if let Some(item) = s.get("items") {
            for (i, v) in items.iter().enumerate() {
                check(root, item, v, &format!("{path}[{i}]"), errors);
            }
        }
        let len = items.len() as u64;
        if s.get("minItems").and_then(Value::as_u64).is_some_and(|m| len < m) {
            errors.push(format!("{path}: {len} items is too few"));
        }
        if s.get("maxItems").and_then(Value::as_u64).is_some_and(|m| len > m) {
            errors.push(format!("{path}: {len} items is too many"));
        }
    }
    if let (Some(n), Some(min)) = (value.as_f64(), s.get("minimum").and_then(Value::as_f64))
        && n < min
    {
        errors.push(format!("{path}: {n} is below {min}"));
    }
}

fn validate_at(root: &Value, schema: &Value, value: &Value) -> bool {
    let mut e = Vec::new();
    check(root, schema, value, "", &mut e);
    e.is_empty()
}

fn is_type(v: &Value, t: &str) -> bool {
    match t {
        "null" => v.is_null(),
        "boolean" => v.is_boolean(),
        "object" => v.is_object(),
        "array" => v.is_array(),
        "string" => v.is_string(),
        "number" => v.is_number(),
        "integer" => v.as_f64().is_some_and(|n| n.fract() == 0.0),
        _ => false,
    }
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
