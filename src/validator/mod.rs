use oca_ast_semantics::ast::{AttributeType, NestedAttrType};
use oca_bundle_semantics::state::{
    attribute::Attribute, entry_codes::EntryCodes, oca::{OCABox, OCABundle}
};

pub fn validate(oca: &OCABundle, data: &str) -> Result<Vec<String>, String> {
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

    oca_box.attributes.values().for_each(|attr| {
        let value = d.get(attr.name.clone());
        let attribute_errors = validate_attribute(attr, value).unwrap();

        if !attribute_errors.is_empty() {
            errors.extend(attribute_errors);
        }
    });

    Ok(errors)
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
            },
            EntryCodes::Object(codes) => {
                if !codes.values().any(|c| c.contains(&v.as_str().unwrap().to_string())) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use oca_bundle_semantics::controller::load_oca;

    #[test]
    fn it_works() {
        let oca_str = r#"{
  "v": "OCAS11JSON001920_",
  "d": "EP0QSau9GRr8uDXaSNR_QCWsURhUyv8Y0gMtqGuNRMdk",
  "capture_base": {
    "d": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
    "type": "spec/capture_base/1.0",
    "attributes": {
      "add_optional_customer": "Boolean",
      "customer": "refs:ELGWVXrWMV-VE2FFvpBUnc1kitYaqdjQkt0fc548Rdci",
      "d": "Text",
      "date": "DateTime",
      "devices": [
        "refs:EC8R1ohDQn3ceXCn2SAMdn3lmDlfudj09JI5ReyW-cdH"
      ],
      "i": "Text",
      "img": "Binary",
      "isbn": "Text",
      "list_bool": [
        "Boolean"
      ],
      "list_date": [
        "DateTime"
      ],
      "list_file": [
        "Binary"
      ],
      "list_num": [
        "Numeric"
      ],
      "list_question": [
        "refs:EPx6U76rCHy_D008Nez5gbE3NOcZW3V0N59DLM2COeZS"
      ],
      "list_sign": [
        "Binary"
      ],
      "list_text": [
        "Text"
      ],
      "list_text2": [
        "Text"
      ],
      "list_text3": [
        "Text"
      ],
      "nice_attr": "Text",
      "num": "Numeric",
      "num2": "Numeric",
      "num3": "Numeric",
      "optional_customer": "refs:EPH3m3t9TZ3NcVZvoyFweRAN1YnMhNSGAEpKUefl7oAO",
      "passed": "Boolean",
      "question1": "refs:EPx6U76rCHy_D008Nez5gbE3NOcZW3V0N59DLM2COeZS",
      "radio1": "Text",
      "radio2": "Text",
      "radio3": "Boolean",
      "select": "Text",
      "selectmulti": [
        "Text"
      ],
      "sign": "Binary",
      "sign_with_geolocation": "refs:ELCZFawLrtXg-Cvh5VQ2gKC9aVy15tZpmmCDyFKx2OC9",
      "text_attr1": "Text",
      "text_attr2": "Text",
      "time": "DateTime"
    },
    "classification": "",
    "flagged_attributes": []
  },
  "overlays": {
    "cardinality": {
      "d": "EMjo0EWYoX0VvckiYBM0_apjDFpDyY1Qg1tgsluA4xMA",
      "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
      "type": "spec/overlays/cardinality/1.0",
      "attribute_cardinality": {
        "devices": "1-",
        "list_file": "-4",
        "list_text": "1-",
        "list_text2": "2-",
        "list_text3": "2"
      }
    },
    "character_encoding": {
      "d": "EAc70d6WcN1FgNm91bp20PoYRDkOoLfmKOwBJ8_lzLSW",
      "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
      "type": "spec/overlays/character_encoding/1.0",
      "attribute_character_encoding": {
        "d": "utf-8",
        "i": "utf-8",
        "nice_attr": "utf-8",
        "passed": "utf-8"
      }
    },
    "conformance": {
      "d": "ELeGs9-SXLTta7S_1zrqs_zowlulJTY5GPuDHLlAMC2f",
      "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
      "type": "spec/overlays/conformance/1.0",
      "attribute_conformance": {
        "d": "M",
        "i": "M",
        "nice_attr": "O",
        "passed": "M",
        "radio3": "M",
        "select": "M",
        "selectmulti": "M"
      }
    },
    "entry": [
      {
        "d": "EATTKqXXqjqAtAkPtk6LuQpJ_Ltrl8mkGXcU6L_DEWl_",
        "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
        "type": "spec/overlays/entry/1.0",
        "language": "eng",
        "attribute_entries": {
          "radio1": {
            "o1": "Jeden",
            "o2": "Dwa",
            "o3": "Trzy"
          },
          "radio2": {
            "o1": "Jeden",
            "o2": "Dwa",
            "o3": "Trzy",
            "o4": "Cztery",
            "o5": "Pięć",
            "o6": "Sześć"
          },
          "select": {
            "g1": "group 1",
            "g2": "group 2",
            "o1": "o1_label",
            "o2": "o2_label",
            "o3": "o3_label"
          },
          "selectmulti": {
            "g3": "Group 3",
            "g4": "Group 4",
            "o4": "Four",
            "o5": "Five",
            "o6": "Six",
            "o7": "Seven"
          }
        }
      },
      {
        "d": "EFzqQsYAsrN7WMnm5hayuiliXOvaTgqcA9i-TVLtsSY1",
        "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
        "type": "spec/overlays/entry/1.0",
        "language": "pol",
        "attribute_entries": {
          "select": {
            "g1": "grupa 1",
            "g2": "grupa 2",
            "o1": "o1_etykieta",
            "o2": "o2_etykieta",
            "o3": "o3_etykieta"
          },
          "selectmulti": {
            "g3": "Grupa 3",
            "g4": "Grupa 4",
            "o4": "Cztery",
            "o5": "Pięc",
            "o6": "Sześć",
            "o7": "Siedem"
          }
        }
      }
    ],
    "entry_code": {
      "d": "EA1BkJWi6Hn-6UcDgcl8jBJ4UnNALPvZuUliEnvC_fgO",
      "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
      "type": "spec/overlays/entry_code/1.0",
      "attribute_entry_codes": {
        "radio1": [
          "o1",
          "o2",
          "o3"
        ],
        "radio2": [
          "o1",
          "o2",
          "o3",
          "o4",
          "o5",
          "o6"
        ],
        "select": {
          "g1": [
            "o1"
          ],
          "g2": [
            "o2",
            "o3"
          ]
        },
        "selectmulti": {
          "g3": [
            "o4",
            "o5"
          ],
          "g4": [
            "o6",
            "o7"
          ]
        }
      }
    },
    "format": {
      "d": "ELE13oR5oX-dJLoSkd3EsDRm8riaN_i79haN5uu2yCTu",
      "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
      "type": "spec/overlays/format/1.0",
      "attribute_formats": {
        "date": "DD.MM.YYYY",
        "i": "^issuer[0-9]+$",
        "list_date": "DD/MM/YYYY (HH:mm:ss)",
        "time": "hh:mm A"
      }
    },
    "information": [
      {
        "d": "EDAQjkgQbXuo4cLq5_6OiRhhet7cDxi4ezhSi1Jrnzry",
        "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
        "type": "spec/overlays/information/1.0",
        "language": "eng",
        "attribute_information": {
          "d": "Schema digest",
          "i": "Credential Issuee",
          "nice_attr": "nice placeholder",
          "passed": "Enables or disables passing",
          "select": "Select option",
          "selectmulti": "choose multi option"
        }
      }
    ],
    "label": [
      {
        "d": "EC4yk20ZHRG1lOd_TMKEKhWDMzlMdCc5itXHr0ozixmh",
        "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
        "type": "spec/overlays/label/1.0",
        "language": "pol",
        "attribute_categories": [],
        "attribute_labels": {
          "add_optional_customer": "Dodaj klienta",
          "customer": "Klient",
          "d": "trawić schemat",
          "devices": "Urządzenia",
          "i": "Kredenszjal wystawiacz",
          "img": "Obrazek",
          "isbn": "ISBN",
          "list_question": "Pytania",
          "nice_attr": "Ładny atrybut",
          "optional_customer": "Klient",
          "passed": "Zaliczony",
          "question1": "Pytanie 1",
          "radio1": "Radio guzik pionowy",
          "radio2": "Radio guzik poziomy",
          "radio3": "Radio bulin",
          "select": "Wybierz opcję",
          "selectmulti": "Multiselekt",
          "sign": "Podpis",
          "sign_with_geolocation": "Podpis",
          "text_attr1": "Pojazd",
          "text_attr2": "Koło"
        },
        "category_labels": {}
      },
      {
        "d": "EOXuYmoxvoyHEZkGUWAIv-2wgx8b2AbvJ7ecBBzHJp4w",
        "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
        "type": "spec/overlays/label/1.0",
        "language": "eng",
        "attribute_categories": [],
        "attribute_labels": {
          "add_optional_customer": "Add customer",
          "customer": "Customer",
          "d": "Schema digest",
          "date": "Date",
          "devices": "Devices",
          "i": "Credential Issuee",
          "img": "Image",
          "isbn": "ISBN",
          "list_bool": "List (bool)",
          "list_date": "List (date)",
          "list_file": "List (file)",
          "list_num": "List (numeric)",
          "list_question": "Questions",
          "list_sign": "List (sign)",
          "list_text": "Text list1",
          "list_text2": "Text list2",
          "list_text3": "Text list3",
          "nice_attr": "Nice attribute",
          "num": "Number",
          "num2": "Number2",
          "num3": "Number3",
          "optional_customer": "Customer",
          "passed": "Passed",
          "question1": "Question 1",
          "radio1": "Radio btn vertical",
          "radio2": "Radio btn horizontal",
          "radio3": "Radio boolean",
          "select": "Select option lbl",
          "selectmulti": "Multiselect",
          "sign": "Signature",
          "sign_with_geolocation": "Signature",
          "text_attr1": "Vehicle",
          "text_attr2": "Wheel",
          "time": "Time"
        },
        "category_labels": {}
      }
    ],
    "meta": [
      {
        "d": "EB4VQ9Bqi1eAMELzMXrtALzrM50ACSjqGyoYdqcpTVMI",
        "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
        "type": "spec/overlays/meta/1.0",
        "language": "eng",
        "description": "Entrance credential",
        "name": "Entrance credential"
      },
      {
        "d": "EIwtp8eLiMD0fcIT3FeLfI-ztEuHXYeYEGGkyFxiwz1d",
        "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
        "type": "spec/overlays/meta/1.0",
        "language": "pol",
        "description": "Kredenszjal wejściowy",
        "name": "Kredenszjal wejściowy"
      }
    ],
    "unit": {
      "d": "EPGNto9fSnhF9sbLd_0uuclV60edIV--TWYcJ5vunKwy",
      "capture_base": "EGQbKoJjLEcAfjkg3iW9tpwmnmidqxActG-dB0onEvqw",
      "type": "spec/overlays/unit/1.0",
      "attribute_unit": {
        "list_num": "kg",
        "num": "m"
      }
    }
  }
}"#;

        let captured_data = r#"{
  "i": "issuer1",
  "devices": [
    {
      "name": "device1",
      "description": "desc1",
      "_id": "f7027bf0-9e44-4168-8f62-bd25c61adeae",
      "manufacturer": {
        "name": "dsa",
        "address": {
          "city": "asd",
          "is_nice": "false"
        }
      }
    }
  ],
  "img": "EBoF5bgnlYLxf__l9siWgFgqDwQGGqlp46TEiqym1SJ4",
  "select": "o1",
  "selectmulti": [
    "o5"
  ],
  "list_text": [
    "asd"
  ],
  "list_text2": [
    "dsa",
    "asd"
  ],
  "list_text3": [
    "dsa",
    "asd"
  ],
  "radio3": "false",
  "num": 2
}"#;

        let oca = load_oca(&mut oca_str.as_bytes()).unwrap();
        let result = validate(&oca, captured_data).unwrap();
        assert_eq!(result.len(), 3);
    }
}
