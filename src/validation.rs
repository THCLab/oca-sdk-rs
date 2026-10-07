//! Validation of data against OCA bundles.
//!
//! [`Validator`] checks a JSON value against a bundle and every bundle it
//! references, and reports each problem with the path of the value at fault.
//!
//! What is checked:
//!
//! - **Capture base**: every attribute's type (`Text`, `Numeric`, `Boolean`,
//!   `DateTime`, `Binary`), arrays at any depth, and references (`refs:<SAID>`)
//!   to other bundles, which are validated recursively. Attributes the bundle
//!   does not declare are errors unless [`Options::allow_unknown_attributes`].
//! - **Core overlays**: `conformance` (`M` attributes must be present),
//!   `entry_code` (the value must be one of the codes), `format` (a `Text`
//!   value must match the regular expression in full) and `cardinality` (the
//!   number of items of an array, or entries of a map: `n`, `n-`, `-m`, `n-m`).
//! - **Community structural overlays** (`hcf:`, see the overlays repository):
//!   `variant`, `range`, `length`, `map`, `recursive`, `union` and `json`.
//!
//! Overlays that carry no rule about values (meta, label, information,
//! character encoding, unit, standard, sensitive, mapping, ...) are not
//! checked. Overlays this module does not know are ignored; a caller can apply
//! its own through a [`Visitor`], which sees every object together with the
//! bundle it was validated against.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use oca_ast::ast::{AttributeType, NestedAttrType, RefValue};
/// Values of overlay properties, as [`overlay`] returns them.
pub use oca_ast::ast::NestedValue;
use oca_bundle::state::oca_bundle::OCABundleModel;
use overlay_file::overlay_registry::OverlayLocalRegistry;
use regex::Regex;
use said::SelfAddressingIdentifier;
use serde_json::{Map, Value};

// ─── Bundle set ───────────────────────────────────────────────────────────────

/// Bundles by SAID, so references between them can be followed.
#[derive(Default, Clone)]
pub struct BundleSet {
    bundles: HashMap<String, OCABundleModel>,
}

impl BundleSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load serialized bundles (OCA bundle JSON). `registry` must define every
    /// overlay they use.
    pub fn from_json<'a>(
        bundles: impl IntoIterator<Item = &'a str>,
        registry: &OverlayLocalRegistry,
    ) -> Result<Self, String> {
        let mut set = Self::new();
        for json in bundles {
            let bundle = oca_bundle::controller::load_oca(&mut json.as_bytes(), registry)
                .map_err(|e| format!("cannot load bundle: {e}"))?;
            set.insert(bundle)?;
        }
        Ok(set)
    }

    /// Add a bundle. It must have a digest.
    pub fn insert(&mut self, bundle: OCABundleModel) -> Result<(), String> {
        let said = bundle
            .digest
            .as_ref()
            .ok_or("bundle has no digest")?
            .to_string();
        self.bundles.insert(said, bundle);
        Ok(())
    }

    pub fn get(&self, said: &str) -> Option<&OCABundleModel> {
        self.bundles.get(said.strip_prefix("refs:").unwrap_or(said))
    }

    pub fn len(&self) -> usize {
        self.bundles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bundles.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &OCABundleModel)> {
        self.bundles.iter()
    }
}

// ─── Results ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// The value should be an object (a bundle's data).
    NotAnObject,
    /// The object has an attribute its bundle does not declare.
    UnknownAttribute,
    /// A mandatory attribute (conformance `M`) is missing.
    MissingAttribute,
    /// The value is not of the attribute's type.
    WrongType,
    /// The value is not one of the attribute's entry codes.
    NotAnEntryCode,
    /// A text value does not match the attribute's format.
    FormatMismatch,
    /// A number is outside the attribute's range.
    OutOfRange,
    /// A text is shorter or longer than the attribute allows.
    BadLength,
    /// An array or map has fewer or more items than its cardinality allows.
    Cardinality,
    /// A key of a map does not match the allowed key pattern.
    BadMapKey,
    /// A recursive attribute nests deeper than allowed.
    TooDeep,
    /// A text that should hold JSON does not parse.
    InvalidJson,
    /// A reference names a bundle that is not in the set.
    UnresolvedReference,
    /// The bundle itself is unusable here (an invalid pattern or range).
    InvalidSchema,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Where the problem is: `$` for the root, then `.name` and `[index]`.
    pub path: String,
    pub kind: ErrorKind,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// Sees every object as it is validated, with the bundle it is validated
/// against (for an object with a variant: first its bundle, then the
/// variant's). Used to apply rules this module does not know.
pub trait Visitor {
    fn visit(&mut self, path: &str, bundle: &OCABundleModel, object: &Map<String, Value>);
}

impl<F: FnMut(&str, &OCABundleModel, &Map<String, Value>)> Visitor for F {
    fn visit(&mut self, path: &str, bundle: &OCABundleModel, object: &Map<String, Value>) {
        self(path, bundle, object)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Accept attributes the bundle does not declare.
    pub allow_unknown_attributes: bool,
}

// ─── Reading overlays ─────────────────────────────────────────────────────────

/// The properties of an overlay.
pub type Props = indexmap::IndexMap<String, NestedValue>;

/// The properties of the bundle's overlay named `name` (`conformance`,
/// `hcf:range`, ...), whatever its version.
pub fn overlay<'b>(bundle: &'b OCABundleModel, name: &str) -> Option<&'b Props> {
    bundle
        .overlays
        .iter()
        .find(|o| {
            let full = o.name.strip_prefix("overlay/").unwrap_or(&o.name);
            let base = full.rsplit_once('/').map(|(n, _)| n).unwrap_or(full);
            base.eq_ignore_ascii_case(name)
        })
        .and_then(|o| o.properties.as_ref())
}

/// The text of an overlay property: `props[element]` when it is text.
pub fn text<'b>(props: Option<&'b Props>, element: &str) -> Option<&'b str> {
    match props?.get(element)? {
        NestedValue::Value(s) => Some(s),
        _ => None,
    }
}

/// The entry of an attribute in an overlay's per-attribute object: `props[element][attr]`.
pub fn attr_entry<'b>(
    props: Option<&'b Props>,
    element: &str,
    attr: &str,
) -> Option<&'b NestedValue> {
    match props?.get(element)? {
        NestedValue::Object(map) => map.get(attr),
        _ => None,
    }
}

fn attr_text<'b>(props: Option<&'b Props>, element: &str, attr: &str) -> Option<&'b str> {
    match attr_entry(props, element, attr)? {
        NestedValue::Value(s) => Some(s),
        _ => None,
    }
}

fn attr_ref(props: Option<&Props>, element: &str, attr: &str) -> Option<String> {
    match attr_entry(props, element, attr)? {
        NestedValue::Reference(RefValue::Said(s)) => Some(s.to_string()),
        NestedValue::Reference(RefValue::Name(n)) => Some(format!("refn:{n}")),
        NestedValue::Value(s) => Some(s.clone()),
        _ => None,
    }
}

/// The rules one bundle puts on its attributes, read once per object.
struct Rules<'b> {
    conformance: Option<&'b Props>,
    entry_code: Option<&'b Props>,
    format: Option<&'b Props>,
    cardinality: Option<&'b Props>,
    range: Option<&'b Props>,
    length: Option<&'b Props>,
    map: Option<&'b Props>,
    recursive: Option<&'b Props>,
    union: Option<&'b Props>,
    json: Option<&'b Props>,
}

impl<'b> Rules<'b> {
    fn of(b: &'b OCABundleModel) -> Self {
        Rules {
            conformance: overlay(b, "conformance"),
            entry_code: overlay(b, "entry_code"),
            format: overlay(b, "format"),
            cardinality: overlay(b, "cardinality"),
            range: overlay(b, "hcf:range"),
            length: overlay(b, "hcf:length"),
            map: overlay(b, "hcf:map"),
            recursive: overlay(b, "hcf:recursive"),
            union: overlay(b, "hcf:union"),
            json: overlay(b, "hcf:json"),
        }
    }
}

// ─── Validator ────────────────────────────────────────────────────────────────

pub struct Validator<'s> {
    set: &'s BundleSet,
    options: Options,
    patterns: RefCell<HashMap<String, Result<Regex, String>>>,
}

/// A bound such as `1..6`, `0..`, `..3`, or a cardinality `1-`, `2-4`, `3`.
#[derive(Debug, Clone, Copy)]
struct Bound {
    min: Option<f64>,
    max: Option<f64>,
}

impl Bound {
    fn range(s: &str) -> Option<Self> {
        let (lo, hi) = s.split_once("..")?;
        Self::from_parts(lo, hi)
    }

    fn cardinality(s: &str) -> Option<Self> {
        match s.split_once('-') {
            Some((lo, hi)) => Self::from_parts(lo, hi),
            None => {
                let n = s.trim().parse().ok()?;
                Some(Bound {
                    min: Some(n),
                    max: Some(n),
                })
            }
        }
    }

    fn from_parts(lo: &str, hi: &str) -> Option<Self> {
        let num = |p: &str| -> Option<Option<f64>> {
            let p = p.trim();
            if p.is_empty() {
                Some(None)
            } else {
                p.parse().ok().map(Some)
            }
        };
        Some(Bound {
            min: num(lo)?,
            max: num(hi)?,
        })
    }

    fn contains(&self, x: f64) -> bool {
        self.min.is_none_or(|m| x >= m) && self.max.is_none_or(|m| x <= m)
    }
}

const ISO_DATE: &str =
    r"^\d{4}-\d{2}-\d{2}([T ]\d{2}:\d{2}(:\d{2}(\.\d+)?)?(Z|[+-]\d{2}:?\d{2})?)?$";
const DECIMAL: &str = r"^-?\d+(\.\d+)?$";

struct Ctx<'v, 'w> {
    errors: &'w mut Vec<ValidationError>,
    visitor: Option<&'v mut dyn Visitor>,
}

impl<'s> Validator<'s> {
    pub fn new(set: &'s BundleSet) -> Self {
        Self::with_options(set, Options::default())
    }

    pub fn with_options(set: &'s BundleSet, options: Options) -> Self {
        Validator {
            set,
            options,
            patterns: RefCell::new(HashMap::new()),
        }
    }

    /// Validate `data` against the bundle with SAID `bundle`.
    pub fn validate(&self, data: &Value, bundle: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        self.run(data, bundle, &mut errors, None);
        errors
    }

    /// Like [`Validator::validate`], calling `visitor` for every object.
    pub fn validate_with(
        &self,
        data: &Value,
        bundle: &str,
        visitor: &mut dyn Visitor,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        self.run(data, bundle, &mut errors, Some(visitor));
        errors
    }

    fn run(
        &self,
        data: &Value,
        bundle: &str,
        errors: &mut Vec<ValidationError>,
        visitor: Option<&mut dyn Visitor>,
    ) {
        let mut ctx = Ctx { errors, visitor };
        match self.set.get(bundle) {
            Some(b) => self.object(data, b, "$", 0, &mut ctx),
            None => ctx.errors.push(err(
                "$",
                ErrorKind::UnresolvedReference,
                format!("no bundle {bundle} in the set"),
            )),
        }
    }

    fn pattern(&self, p: &str) -> Result<Regex, String> {
        self.patterns
            .borrow_mut()
            .entry(p.to_string())
            // Formats match the whole value.
            .or_insert_with(|| Regex::new(&format!("^(?:{p})$")).map_err(|e| e.to_string()))
            .clone()
    }

    fn object(&self, value: &Value, b: &OCABundleModel, path: &str, depth: usize, ctx: &mut Ctx) {
        let Some(obj) = value.as_object() else {
            ctx.errors.push(err(
                path,
                ErrorKind::NotAnObject,
                format!("{} is not an object", short(value)),
            ));
            return;
        };
        let rules = Rules::of(b);

        // The shape of this object may be chosen by a discriminator.
        let variant = overlay(b, "hcf:variant").and_then(|v| {
            let disc = text(Some(v), "discriminator")?;
            let code = match obj.get(disc)? {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            match v.get("variants")? {
                NestedValue::Object(m) => m.get(&code).map(|r| match r {
                    NestedValue::Reference(RefValue::Said(s)) => s.to_string(),
                    NestedValue::Reference(RefValue::Name(n)) => format!("refn:{n}"),
                    NestedValue::Value(s) => s.clone(),
                    _ => String::new(),
                }),
                _ => None,
            }
        });

        // The variant's bundle decides which attributes are allowed.
        if variant.is_none() && !self.options.allow_unknown_attributes {
            for k in obj.keys() {
                if !b.capture_base.attributes.contains_key(k) {
                    ctx.errors.push(err(
                        &format!("{path}.{k}"),
                        ErrorKind::UnknownAttribute,
                        format!("{k} is not an attribute of {}", bundle_name(b)),
                    ));
                }
            }
        }

        for (name, ty) in &b.capture_base.attributes {
            let at = format!("{path}.{name}");
            let Some(v) = obj.get(name) else {
                if attr_text(rules.conformance, "attribute_conformances", name) == Some("M") {
                    ctx.errors.push(err(
                        &at,
                        ErrorKind::MissingAttribute,
                        format!("{name} is mandatory"),
                    ));
                }
                continue;
            };

            if let Some(max) = attr_text(rules.recursive, "attribute_recursions", name) {
                let Some(items) = v.as_array() else {
                    ctx.errors.push(err(
                        &at,
                        ErrorKind::WrongType,
                        format!("{} is not an array", short(v)),
                    ));
                    continue;
                };
                self.cardinality(&rules, name, items.len(), &at, ctx);
                match max.trim().parse::<usize>() {
                    Ok(max) if depth >= max => {
                        ctx.errors.push(err(
                            &at,
                            ErrorKind::TooDeep,
                            format!("nested deeper than {max}"),
                        ));
                    }
                    Ok(_) => {
                        for (i, item) in items.iter().enumerate() {
                            self.object(item, b, &format!("{at}[{i}]"), depth + 1, ctx);
                        }
                    }
                    Err(_) => ctx.errors.push(err(
                        &at,
                        ErrorKind::InvalidSchema,
                        format!("depth {max:?} is not a number"),
                    )),
                }
                continue;
            }

            if let Some(keys) = attr_text(rules.map, "attribute_maps", name) {
                let Some(m) = v.as_object() else {
                    ctx.errors.push(err(
                        &at,
                        ErrorKind::WrongType,
                        format!("{} is not a map", short(v)),
                    ));
                    continue;
                };
                self.cardinality(&rules, name, m.len(), &at, ctx);
                let keys = self.pattern(keys);
                for (k, x) in m {
                    let p = format!("{at}.{k}");
                    match &keys {
                        Ok(re) if !re.is_match(k) => ctx.errors.push(err(
                            &p,
                            ErrorKind::BadMapKey,
                            format!("key {k:?} does not match {}", re.as_str()),
                        )),
                        Err(e) => ctx.errors.push(err(
                            &at,
                            ErrorKind::InvalidSchema,
                            format!("key pattern: {e}"),
                        )),
                        _ => {}
                    }
                    self.value(x, ty, name, &rules, &p, depth, ctx);
                }
                continue;
            }

            if let (NestedAttrType::Array(_), Some(items)) = (ty, v.as_array()) {
                self.cardinality(&rules, name, items.len(), &at, ctx);
            }
            self.value(v, ty, name, &rules, &at, depth, ctx);
        }

        if let Some(v) = ctx.visitor.as_mut() {
            v.visit(path, b, obj);
        }

        if let Some(target) = variant {
            match self.set.get(&target) {
                Some(vb) => self.object(value, vb, path, depth, ctx),
                None => ctx.errors.push(err(
                    path,
                    ErrorKind::UnresolvedReference,
                    format!("variant {target} is not in the set"),
                )),
            }
        }
    }

    fn cardinality(&self, rules: &Rules, name: &str, n: usize, at: &str, ctx: &mut Ctx) {
        let Some(spec) = attr_text(rules.cardinality, "attribute_cardinalities", name) else {
            return;
        };
        match Bound::cardinality(spec) {
            Some(b) if !b.contains(n as f64) => ctx.errors.push(err(
                at,
                ErrorKind::Cardinality,
                format!("{n} items, allowed {spec}"),
            )),
            None => ctx.errors.push(err(
                at,
                ErrorKind::InvalidSchema,
                format!("cardinality {spec:?} cannot be read"),
            )),
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn value(
        &self,
        v: &Value,
        ty: &NestedAttrType,
        name: &str,
        rules: &Rules,
        at: &str,
        depth: usize,
        ctx: &mut Ctx,
    ) {
        // A union replaces the declared type of the value (of the items, for arrays).
        let union = attr_text(rules.union, "attribute_unions", name);
        match ty {
            NestedAttrType::Array(inner) => {
                let Some(items) = v.as_array() else {
                    ctx.errors.push(err(
                        at,
                        ErrorKind::WrongType,
                        format!("{} is not an array", short(v)),
                    ));
                    return;
                };
                for (i, item) in items.iter().enumerate() {
                    self.value(item, inner, name, rules, &format!("{at}[{i}]"), depth, ctx);
                }
            }
            NestedAttrType::Reference(RefValue::Said(said)) => {
                match self.set.get(&said.to_string()) {
                    Some(b) => self.object(v, b, at, depth + 1, ctx),
                    None => ctx.errors.push(err(
                        at,
                        ErrorKind::UnresolvedReference,
                        format!("bundle {said} is not in the set"),
                    )),
                }
            }
            NestedAttrType::Reference(RefValue::Name(n)) => {
                ctx.errors.push(err(
                    at,
                    ErrorKind::UnresolvedReference,
                    format!("refn:{n} is a name, not a SAID"),
                ));
            }
            NestedAttrType::Null => {}
            NestedAttrType::Value(t) => {
                let allowed: Vec<&str> = match union {
                    Some(u) => u.split('|').map(str::trim).collect(),
                    None => vec![type_name(t)],
                };
                if v.is_null() {
                    if !allowed.iter().any(|a| a.eq_ignore_ascii_case("Null")) {
                        ctx.errors.push(err(
                            at,
                            ErrorKind::WrongType,
                            format!("null is not {}", allowed.join("|")),
                        ));
                    }
                    return;
                }
                if !allowed.iter().any(|a| self.is_type(a, v)) {
                    ctx.errors.push(err(
                        at,
                        ErrorKind::WrongType,
                        format!("{} is not {}", short(v), allowed.join("|")),
                    ));
                    return;
                }
                self.leaf(v, name, rules, at, depth, ctx);
            }
        }
    }

    fn is_type(&self, t: &str, v: &Value) -> bool {
        match t.to_ascii_lowercase().as_str() {
            "text" | "binary" => v.is_string(),
            "numeric" => v.is_number(),
            "boolean" => v.is_boolean(),
            "datetime" => v
                .as_str()
                .is_some_and(|s| self.pattern(ISO_DATE).is_ok_and(|re| re.is_match(s))),
            "null" => v.is_null(),
            _ => false,
        }
    }

    fn leaf(&self, v: &Value, name: &str, rules: &Rules, at: &str, depth: usize, ctx: &mut Ctx) {
        if let Some(NestedValue::Array(codes)) =
            attr_entry(rules.entry_code, "attribute_entry_codes", name)
        {
            let as_text = match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let ok = codes
                .iter()
                .any(|c| matches!(c, NestedValue::Value(s) if *s == as_text));
            if !ok {
                let list: Vec<&str> = codes
                    .iter()
                    .filter_map(|c| {
                        if let NestedValue::Value(s) = c {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .collect();
                ctx.errors.push(err(
                    at,
                    ErrorKind::NotAnEntryCode,
                    format!("{} is not one of {}", short(v), list.join(", ")),
                ));
            }
        }

        if let (Some(s), Some(f)) = (
            v.as_str(),
            attr_text(rules.format, "attribute_formats", name),
        ) {
            match self.pattern(f) {
                Ok(re) if !re.is_match(s) => ctx.errors.push(err(
                    at,
                    ErrorKind::FormatMismatch,
                    format!("{} does not match {f}", short(v)),
                )),
                Err(e) => ctx.errors.push(err(
                    at,
                    ErrorKind::InvalidSchema,
                    format!("format {f:?}: {e}"),
                )),
                _ => {}
            }
        }

        if let Some(spec) = attr_text(rules.range, "attribute_ranges", name) {
            let n = match v {
                Value::Number(n) => n.as_f64(),
                Value::String(s) if self.pattern(DECIMAL).is_ok_and(|re| re.is_match(s.trim())) => {
                    f64::from_str(s.trim()).ok()
                }
                _ => None,
            };
            match (Bound::range(spec), n) {
                (None, _) => ctx.errors.push(err(
                    at,
                    ErrorKind::InvalidSchema,
                    format!("range {spec:?} cannot be read"),
                )),
                (Some(_), None) => ctx.errors.push(err(
                    at,
                    ErrorKind::OutOfRange,
                    format!("{} is not a number", short(v)),
                )),
                (Some(b), Some(x)) if !b.contains(x) => ctx.errors.push(err(
                    at,
                    ErrorKind::OutOfRange,
                    format!("{} is outside {spec}", short(v)),
                )),
                _ => {}
            }
        }

        if let (Some(s), Some(spec)) = (
            v.as_str(),
            attr_text(rules.length, "attribute_lengths", name),
        ) {
            let n = s.chars().count();
            match Bound::range(spec) {
                Some(b) if !b.contains(n as f64) => ctx.errors.push(err(
                    at,
                    ErrorKind::BadLength,
                    format!("{n} characters, allowed {spec}"),
                )),
                None => ctx.errors.push(err(
                    at,
                    ErrorKind::InvalidSchema,
                    format!("length {spec:?} cannot be read"),
                )),
                _ => {}
            }
        }

        if let (Some(s), Some(target)) = (v.as_str(), attr_ref(rules.json, "attribute_json", name))
        {
            match serde_json::from_str::<Value>(s) {
                Ok(decoded) => match self.set.get(&target) {
                    Some(b) => self.object(&decoded, b, &format!("{at}<json>"), depth + 1, ctx),
                    None => ctx.errors.push(err(
                        at,
                        ErrorKind::UnresolvedReference,
                        format!("bundle {target} is not in the set"),
                    )),
                },
                Err(e) => {
                    ctx.errors
                        .push(err(at, ErrorKind::InvalidJson, format!("not JSON: {e}")))
                }
            }
        }
    }
}

fn type_name(t: &AttributeType) -> &'static str {
    match t {
        AttributeType::Text => "Text",
        AttributeType::Numeric => "Numeric",
        AttributeType::Boolean => "Boolean",
        AttributeType::DateTime => "DateTime",
        AttributeType::Binary => "Binary",
    }
}

fn err(path: &str, kind: ErrorKind, message: String) -> ValidationError {
    ValidationError {
        path: path.to_string(),
        kind,
        message,
    }
}

fn short(v: &Value) -> String {
    let s = v.to_string();
    if s.chars().count() > 40 {
        format!("{}…", s.chars().take(40).collect::<String>())
    } else {
        s
    }
}

fn bundle_name(b: &OCABundleModel) -> String {
    let name = text(overlay(b, "meta"), "name");
    match (name, &b.digest) {
        (Some(n), _) => n.to_string(),
        (None, Some(d)) => d.to_string(),
        _ => "the bundle".to_string(),
    }
}

/// Parse a SAID, for callers that hold one as text.
pub fn said(s: &str) -> Result<SelfAddressingIdentifier, String> {
    SelfAddressingIdentifier::from_str(s.strip_prefix("refs:").unwrap_or(s))
        .map_err(|e| format!("{e:?}"))
}
