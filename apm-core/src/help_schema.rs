use schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};
use schemars::JsonSchema;

pub struct FieldEntry {
    pub toml_path: String,
    pub type_name: String,
    pub default: Option<String>,
    pub description: Option<String>,
    pub enum_variants: Option<Vec<String>>,
    pub required: bool,
}

pub fn schema_entries<T: JsonSchema>() -> Vec<FieldEntry> {
    let root = schemars::schema_for!(T);
    let defs = &root.definitions;
    walk_object(&root.schema, defs, "")
}

pub fn render_schema<T: JsonSchema>() -> String {
    let entries = schema_entries::<T>();
    if entries.is_empty() {
        return String::new();
    }
    let path_w = entries.iter().map(|e| e.toml_path.len()).max().unwrap_or(0);
    let type_w = entries.iter().map(|e| e.type_name.len()).max().unwrap_or(0);
    entries
        .iter()
        .map(|e| {
            let mut line =
                format!("{:<path_w$}  {:<type_w$}", e.toml_path, e.type_name);
            if let Some(ref d) = e.default {
                line.push_str(&format!("  [default: {}]", d));
            }
            if let Some(ref desc) = e.description {
                line.push_str(&format!("  # {}", desc));
            }
            if let Some(ref variants) = e.enum_variants {
                line.push_str(&format!("  ({})", variants.join(" | ")));
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ── internals ──────────────────────────────────────────────────────────────

fn walk_object(
    schema: &SchemaObject,
    defs: &schemars::Map<String, Schema>,
    prefix: &str,
) -> Vec<FieldEntry> {
    let Some(ref obj) = schema.object else {
        return vec![];
    };
    let required_set = &obj.required;

    let mut props: Vec<(&String, &Schema)> = obj.properties.iter().collect();
    props.sort_by_key(|(k, _)| k.as_str());

    let mut result = Vec::new();
    for (field_name, field_schema) in props {
        let Schema::Object(field_obj) = field_schema else {
            continue;
        };

        let path = if prefix.is_empty() {
            field_name.clone()
        } else {
            format!("{}.{}", prefix, field_name)
        };
        let required = required_set.contains(field_name.as_str());

        // Metadata lives on the field schema (wrapper), not the definition.
        let description = field_obj
            .metadata
            .as_ref()
            .and_then(|m| m.description.clone());
        let default_val = field_obj
            .metadata
            .as_ref()
            .and_then(|m| m.default.as_ref())
            .map(fmt_default);

        // Structural information may live in a $ref definition.
        let structural = resolve_structural(field_obj, defs);

        // Fall back to the definition's metadata when the field wrapper has none.
        let description = description.or_else(|| {
            structural
                .metadata
                .as_ref()
                .and_then(|m| m.description.clone())
        });
        let default_val = default_val.or_else(|| {
            structural
                .metadata
                .as_ref()
                .and_then(|m| m.default.as_ref())
                .map(fmt_default)
        });

        let entries = classify(structural, defs, &path, required, description, default_val);
        result.extend(entries);
    }
    result
}

/// Follow a direct `$ref` or an `allOf: [{$ref: ...}]` wrapper to get the
/// schema that carries type structure (instance_type, object, array, etc.).
fn resolve_structural<'a>(
    schema: &'a SchemaObject,
    defs: &'a schemars::Map<String, Schema>,
) -> &'a SchemaObject {
    // Direct $ref
    if let Some(ref r) = schema.reference {
        if let Some(Schema::Object(def)) = defs.get(ref_name(r)) {
            return def;
        }
    }
    // allOf with a single $ref (schemars wraps typed fields that also carry defaults)
    if let Some(ref subs) = schema.subschemas {
        if let Some(ref all_of) = subs.all_of {
            if all_of.len() == 1 {
                if let Schema::Object(inner) = &all_of[0] {
                    if let Some(ref r) = inner.reference {
                        if let Some(Schema::Object(def)) = defs.get(ref_name(r)) {
                            return def;
                        }
                    }
                }
            }
        }
    }
    schema
}

fn ref_name(r: &str) -> &str {
    r.strip_prefix("#/definitions/").unwrap_or(r)
}

fn classify(
    schema: &SchemaObject,
    defs: &schemars::Map<String, Schema>,
    path: &str,
    required: bool,
    description: Option<String>,
    default_val: Option<String>,
) -> Vec<FieldEntry> {
    // ── string enum ────────────────────────────────────────────────────────
    if let Some(ref enum_vals) = schema.enum_values {
        let variants: Vec<String> = enum_vals
            .iter()
            .filter_map(|v| {
                if let serde_json::Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect();
        return vec![FieldEntry {
            toml_path: path.to_string(),
            type_name: "string".to_string(),
            default: default_val,
            description,
            enum_variants: if variants.is_empty() { None } else { Some(variants) },
            required,
        }];
    }

    // ── anyOf / oneOf (untagged enums, Option<T>) ──────────────────────────
    if let Some(ref subs) = schema.subschemas {
        let variants = subs.any_of.as_deref().or(subs.one_of.as_deref());
        if let Some(vs) = variants {
            let non_null: Vec<&Schema> =
                vs.iter().filter(|s| !is_null_schema(s)).collect();

            if non_null.len() == 1 {
                // Option<T> — unwrap and classify the inner type.
                let Schema::Object(inner) = non_null[0] else {
                    return vec![];
                };
                let structural = resolve_structural(inner, defs);
                return classify(structural, defs, path, required, description, default_val);
            } else if non_null.len() > 1 {
                // True union (e.g. SatisfiesDeps: bool | string).
                let type_names: Vec<String> = non_null
                    .iter()
                    .filter_map(|s| {
                        if let Schema::Object(obj) = s {
                            let resolved = resolve_structural(obj, defs);
                            Some(instance_type_name(resolved))
                        } else {
                            None
                        }
                    })
                    .collect();
                return vec![FieldEntry {
                    toml_path: path.to_string(),
                    type_name: type_names.join(" | "),
                    default: default_val,
                    description,
                    enum_variants: None,
                    required,
                }];
            }
        }
    }

    // ── array ──────────────────────────────────────────────────────────────
    if let Some(ref arr) = schema.array {
        if let Some(ref items) = arr.items {
            let item_structural = match items {
                SingleOrVec::Single(s) => {
                    if let Schema::Object(obj) = s.as_ref() {
                        resolve_structural(obj, defs)
                    } else {
                        return vec![];
                    }
                }
                SingleOrVec::Vec(v) => {
                    if let Some(Schema::Object(obj)) = v.first() {
                        resolve_structural(obj, defs)
                    } else {
                        return vec![];
                    }
                }
            };

            let is_struct = item_structural
                .object
                .as_ref()
                .map(|o| !o.properties.is_empty())
                .unwrap_or(false);

            if is_struct {
                return walk_object(item_structural, defs, &format!("{}[]", path));
            } else {
                return vec![FieldEntry {
                    toml_path: path.to_string(),
                    type_name: format!("list-of-{}", instance_type_name(item_structural)),
                    default: default_val,
                    description,
                    enum_variants: None,
                    required,
                }];
            }
        }
        return vec![];
    }

    // ── nested struct or map ───────────────────────────────────────────────
    if let Some(ref obj) = schema.object {
        if !obj.properties.is_empty() {
            // Nested struct — recurse (no FieldEntry for the container).
            return walk_object(schema, defs, path);
        }
        if obj.additional_properties.is_some() {
            // HashMap — emit one entry, do not recurse into values.
            return vec![FieldEntry {
                toml_path: path.to_string(),
                type_name: "map".to_string(),
                default: default_val,
                description,
                enum_variants: None,
                required,
            }];
        }
    }

    // ── scalar ─────────────────────────────────────────────────────────────
    if let Some(ref it) = schema.instance_type {
        let type_name = match it {
            SingleOrVec::Single(t) => scalar_name(t),
            SingleOrVec::Vec(types) => {
                let non_null: Vec<_> = types
                    .iter()
                    .filter(|t| **t != InstanceType::Null)
                    .collect();
                non_null
                    .first()
                    .map(|t| scalar_name(t))
                    .unwrap_or_else(|| "null".to_string())
            }
        };
        return vec![FieldEntry {
            toml_path: path.to_string(),
            type_name,
            default: default_val,
            description,
            enum_variants: None,
            required,
        }];
    }

    vec![]
}

fn is_null_schema(schema: &Schema) -> bool {
    match schema {
        Schema::Object(obj) => matches!(
            &obj.instance_type,
            Some(SingleOrVec::Single(t)) if **t == InstanceType::Null
        ),
        Schema::Bool(_) => false,
    }
}

fn instance_type_name(schema: &SchemaObject) -> String {
    if let Some(ref it) = schema.instance_type {
        match it {
            SingleOrVec::Single(t) => scalar_name(t),
            SingleOrVec::Vec(types) => {
                let non_null: Vec<_> = types
                    .iter()
                    .filter(|t| **t != InstanceType::Null)
                    .collect();
                non_null
                    .first()
                    .map(|t| scalar_name(t))
                    .unwrap_or_else(|| "null".to_string())
            }
        }
    } else {
        "unknown".to_string()
    }
}

fn scalar_name(t: &InstanceType) -> String {
    match t {
        InstanceType::String => "string".to_string(),
        InstanceType::Integer => "integer".to_string(),
        InstanceType::Boolean => "bool".to_string(),
        InstanceType::Number => "number".to_string(),
        InstanceType::Null => "null".to_string(),
        InstanceType::Array => "array".to_string(),
        InstanceType::Object => "object".to_string(),
    }
}

fn fmt_default(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => serde_json::to_string(v).unwrap_or_default(),
    }
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, WorkflowConfig};

    #[test]
    fn agents_max_concurrent_has_default_3() {
        let entries = schema_entries::<Config>();
        let entry = entries
            .iter()
            .find(|e| e.toml_path == "agents.max_concurrent")
            .expect("agents.max_concurrent not found");
        assert_eq!(entry.default.as_deref(), Some("3"));
        assert!(!entry.required);
    }

    #[test]
    fn project_name_is_required() {
        let entries = schema_entries::<Config>();
        let entry = entries
            .iter()
            .find(|e| e.toml_path == "project.name")
            .expect("project.name not found");
        assert!(entry.required);
    }

    #[test]
    fn workflow_states_uses_array_notation() {
        let entries = schema_entries::<Config>();
        assert!(
            entries.iter().any(|e| e.toml_path.starts_with("workflow.states[].")),
            "no entry with toml_path starting with 'workflow.states[].'"
        );
    }

    #[test]
    fn completion_strategy_has_enum_variants() {
        let entries = schema_entries::<Config>();
        let entry = entries
            .iter()
            .find(|e| e.toml_path == "workflow.states[].transitions[].completion")
            .expect("workflow.states[].transitions[].completion not found");
        let variants = entry
            .enum_variants
            .as_ref()
            .expect("enum_variants should be Some");
        assert!(variants.contains(&"none".to_string()), "missing 'none'");
        assert!(variants.contains(&"pr".to_string()), "missing 'pr'");
        assert!(variants.contains(&"merge".to_string()), "missing 'merge'");
        assert!(variants.contains(&"pull".to_string()), "missing 'pull'");
        assert!(
            variants.contains(&"pr_or_epic_merge".to_string()),
            "missing 'pr_or_epic_merge'"
        );
    }

    #[test]
    fn satisfies_deps_has_union_type_name() {
        let entries = schema_entries::<WorkflowConfig>();
        let entry = entries
            .iter()
            .find(|e| e.toml_path == "states[].satisfies_deps")
            .expect("states[].satisfies_deps not found");
        assert_eq!(entry.type_name, "bool | string");
        assert!(entry.enum_variants.is_none());
    }

    #[test]
    fn render_schema_contains_agents_max_concurrent() {
        let output = render_schema::<Config>();
        assert!(!output.is_empty());
        assert!(
            output.contains("agents.max_concurrent"),
            "render_schema output does not contain 'agents.max_concurrent'"
        );
    }
}
