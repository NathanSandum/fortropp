use serde_json::{Map, Value, json};
use thiserror::Error;

const MAX_DEPTH: usize = 64;
const SCHEMA_PREFIX: &str = "#/components/schemas/";
const RESPONSES: [&str; 2] = ["200", "4XX"];
const OPERATIONS: [(&str, &str, &str); 7] = [
    ("dns_list", "/zones/{zone_id}/dns_records", "get"),
    ("dns_create", "/zones/{zone_id}/dns_records", "post"),
    (
        "dns_delete",
        "/zones/{zone_id}/dns_records/{dns_record_id}",
        "delete",
    ),
    ("list", "/zones/{zone_id}/custom_hostnames", "get"),
    ("create", "/zones/{zone_id}/custom_hostnames", "post"),
    (
        "delete",
        "/zones/{zone_id}/custom_hostnames/{custom_hostname_id}",
        "delete",
    ),
    (
        "edit",
        "/zones/{zone_id}/custom_hostnames/{custom_hostname_id}",
        "patch",
    ),
];

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("the spec has no {0}")]
    Missing(String),
    #[error("unsupported reference {0}")]
    UnsupportedReference(String),
    #[error("references nest deeper than {MAX_DEPTH} levels, likely a cycle")]
    TooDeep,
}

fn component<'spec>(spec: &'spec Value, reference: &str) -> Result<&'spec Value, ExtractError> {
    let name = reference
        .strip_prefix(SCHEMA_PREFIX)
        .ok_or_else(|| ExtractError::UnsupportedReference(reference.to_string()))?;
    spec.pointer(&format!("/components/schemas/{name}"))
        .ok_or_else(|| ExtractError::Missing(format!("component schema {name}")))
}

fn nullable(mut object: Map<String, Value>) -> Value {
    if object.remove("nullable") != Some(Value::Bool(true)) {
        return Value::Object(object);
    }
    match object.remove("type") {
        Some(Value::String(kind)) => {
            object.insert("type".to_string(), json!([kind, "null"]));
            Value::Object(object)
        }
        Some(other) => {
            object.insert("type".to_string(), other);
            json!({ "anyOf": [Value::Object(object), { "type": "null" }] })
        }
        None => json!({ "anyOf": [Value::Object(object), { "type": "null" }] }),
    }
}

fn resolve(spec: &Value, node: &Value, depth: usize) -> Result<Value, ExtractError> {
    if depth > MAX_DEPTH {
        return Err(ExtractError::TooDeep);
    }
    let next = depth.saturating_add(1);
    match node {
        Value::Object(object) => {
            if let Some(Value::String(reference)) = object.get("$ref") {
                return resolve(spec, component(spec, reference)?, next);
            }
            let mut resolved = Map::new();
            for (key, value) in object {
                resolved.insert(key.clone(), resolve(spec, value, next)?);
            }
            Ok(nullable(resolved))
        }
        Value::Array(items) => items
            .iter()
            .map(|item| resolve(spec, item, next))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => Ok(node.clone()),
    }
}

pub fn extract(spec: &Value, commit: &str) -> Result<Value, ExtractError> {
    let mut operations = Map::new();
    for (name, path, method) in OPERATIONS {
        let mut responses = Map::new();
        for status in RESPONSES {
            let pointer = format!(
                "/paths/{}/{method}/responses/{status}/content/application~1json/schema",
                path.replace('~', "~0").replace('/', "~1")
            );
            let schema = spec
                .pointer(&pointer)
                .ok_or_else(|| ExtractError::Missing(format!("{method} {path} {status}")))?;
            responses.insert(status.to_string(), resolve(spec, schema, 0)?);
        }
        operations.insert(name.to_string(), Value::Object(responses));
    }
    Ok(json!({
        "source": {
            "repository": "https://github.com/cloudflare/api-schemas",
            "file": "openapi.json",
            "commit": commit,
        },
        "operations": operations,
    }))
}
