use oca_ast_semantics::ast::{AttributeType, NestedAttrType};
use oca_bundle_semantics::state::{
    attribute::Attribute,
    entry_codes::EntryCodes,
    oca::{OCABox, OCABundle},
};

pub enum DataValidationStatus {
    Valid,
    Invalid(Vec<String>)
}

pub fn validate(oca: &OCABundle, data: &str) -> Result<DataValidationStatus, String> {
    let mut errors = vec![];

    let oca_box = OCABox::from(oca.clone());
    let d: serde_json::Value = match serde_json::from_str(data) {
        Ok(d) => d,
        Err(e) => {
            return Err(format!("Failed to parse data: {}", e));
        }
    };

    if !d.is_object() {
        return Err("Data is not an object".to_string());
    }

    for attr in oca_box.attributes.values() {
        let value = d.get(attr.name.clone());
        let attribute_errors = validate_attribute(attr, value)?;

        if !attribute_errors.is_empty() {
            errors.extend(attribute_errors);
        }
    }

    if errors.is_empty() {
        Ok(DataValidationStatus::Valid)
    } else {
        Ok(DataValidationStatus::Invalid(errors))
    }
}

fn validate_attribute(
    attribute: &Attribute,
    value: Option<&serde_json::Value>,
) -> Result<Vec<String>, String> {
    let mut errors = vec![];

    let is_required = attribute.conformance == Some("M".to_string());

    let v = match value {
        Some(value) => value,
        None => {
            if is_required {
                errors.push(format!(
                    "Attribute \"{}\" value is mandatory",
                    attribute.name
                ));
            }
            return Ok(errors);
        }
    };

    if v.is_array() || v.is_object() {
        return Ok(errors);
    }

    if let Some(nested_attribute_type) = &attribute.attribute_type {
        match nested_attribute_type {
            NestedAttrType::Value(attribute_type) => match attribute_type {
                AttributeType::Text => {
                    if !v.is_string() {
                        errors.push(format!(
                            "Attribute \"{}\" value ({}) is not a string",
                            attribute.name, v
                        ));
                    }
                }
                AttributeType::Numeric => {
                    if !v.is_number() {
                        errors.push(format!(
                            "Attribute \"{}\" value ({}) is not a number",
                            attribute.name, v
                        ));
                    }
                }
                AttributeType::DateTime => {
                    if !v.is_string() {
                        errors.push(format!(
                            "Attribute \"{}\" value ({}) is not a string",
                            attribute.name, v
                        ));
                    }
                }
                AttributeType::Boolean => {
                    if !v.is_boolean() {
                        errors.push(format!(
                            "Attribute \"{}\" value ({}) is not a boolean",
                            attribute.name, v
                        ));
                    }
                }
                AttributeType::Binary => {
                    if !v.is_string() {
                        errors.push(format!(
                            "Attribute \"{}\" value ({}) is not a string",
                            attribute.name, v
                        ));
                    }
                }
            },
            NestedAttrType::Array(_) => {
                if !v.is_array() {
                    errors.push(format!(
                        "Attribute \"{}\" value ({}) is not an array",
                        attribute.name, v
                    ));
                }
            }
            NestedAttrType::Null => {}
            _ => {}
        }
    }

    if let Some(entry_codes) = &attribute.entry_codes {
        match entry_codes {
            EntryCodes::Array(codes) => {
                if !codes.contains(&v.as_str().unwrap().to_string()) {
                    errors.push(format!(
                        "Attribute \"{}\" value ({}) is not in entry codes",
                        attribute.name, v
                    ));
                }
            }
            EntryCodes::Object(codes) => {
                if !codes
                    .values()
                    .any(|c| c.contains(&v.as_str().unwrap().to_string()))
                {
                    errors.push(format!(
                        "Attribute \"{}\" value ({}) is not in entry codes",
                        attribute.name, v
                    ));
                }
            }
            _ => {}
        }
    }

    Ok(errors)
}
