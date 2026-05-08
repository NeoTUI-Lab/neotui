// DSL model
// Neutral application spec and parsers for canonical TOML/JSON inputs

use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSpec {
    pub schema_version: String,
    pub theme: Option<String>,
    pub root: ComponentSpec,
}

impl AppSpec {
    pub fn from_toml_str(input: &str) -> Result<Self, DslError> {
        let raw: RawAppSpec =
            toml::from_str(input).map_err(|source| DslError::ParseToml { source })?;
        raw.try_into()
    }

    pub fn from_json_str(input: &str) -> Result<Self, DslError> {
        let raw: RawAppSpec =
            serde_json::from_str(input).map_err(|source| DslError::ParseJson { source })?;
        raw.try_into()
    }

    pub fn validate(&self) -> Result<(), ValidationErrors> {
        DslValidator::new().validate(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSpec {
    pub kind: String,
    pub id: Option<String>,
    pub props: BTreeMap<String, Value>,
    pub children: Vec<ComponentSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(String),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DslFormat {
    Toml,
    Json,
}

impl DslFormat {
    pub fn detect_from_path(path: &str) -> Option<Self> {
        let lower = path.to_ascii_lowercase();

        if lower.ends_with(".toml") {
            Some(Self::Toml)
        } else if lower.ends_with(".json") {
            Some(Self::Json)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum DslError {
    ParseToml { source: toml::de::Error },
    ParseJson { source: serde_json::Error },
    MissingRoot,
    MissingSchemaVersion,
    MissingComponentKind,
}

impl fmt::Display for DslError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseToml { source } => write!(f, "failed to parse TOML DSL: {source}"),
            Self::ParseJson { source } => write!(f, "failed to parse JSON DSL: {source}"),
            Self::MissingRoot => write!(f, "app spec must define a root component"),
            Self::MissingSchemaVersion => write!(f, "app spec must define schema_version"),
            Self::MissingComponentKind => write!(f, "component spec must define kind"),
        }
    }
}

impl std::error::Error for DslError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl ValidationError {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new(errors: Vec<ValidationError>) -> Self {
        Self { errors }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, error) in self.errors.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }

            write!(f, "{error}")?;
        }

        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DslValidator;

impl DslValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, spec: &AppSpec) -> Result<(), ValidationErrors> {
        let mut errors = Vec::new();

        if spec.schema_version.trim().is_empty() {
            errors.push(ValidationError::new(
                "app.schema_version",
                "schema_version must not be empty",
            ));
        }

        if let Some(theme) = &spec.theme {
            if theme.trim().is_empty() {
                errors.push(ValidationError::new(
                    "app.theme",
                    "theme must not be empty when provided",
                ));
            }
        }

        self.validate_component(&spec.root, "root", &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors::new(errors))
        }
    }

    fn validate_component(
        &self,
        component: &ComponentSpec,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        if component.kind.trim().is_empty() {
            errors.push(ValidationError::new(
                format!("{path}.kind"),
                "component kind must not be empty",
            ));
            return;
        }

        if !is_supported_component_kind(&component.kind) {
            errors.push(ValidationError::new(
                format!("{path}.kind"),
                format!("unknown component kind `{}`", component.kind),
            ));
        }

        if let Some(id) = &component.id {
            if id.trim().is_empty() {
                errors.push(ValidationError::new(
                    format!("{path}.id"),
                    "component id must not be empty when provided",
                ));
            }
        }

        self.validate_component_props(component, path, errors);

        for (index, child) in component.children.iter().enumerate() {
            self.validate_component(child, &format!("{path}.children[{index}]"), errors);
        }
    }

    fn validate_component_props(
        &self,
        component: &ComponentSpec,
        path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        match component.kind.as_str() {
            "Label" => {
                validate_required_string_prop(component, path, "text", errors);
                validate_optional_enum_prop(
                    component,
                    path,
                    "align",
                    &["left", "center", "right"],
                    errors,
                );
                validate_no_children(component, path, errors);
            }
            "Divider" => {
                validate_optional_enum_prop(
                    component,
                    path,
                    "orientation",
                    &["horizontal", "vertical"],
                    errors,
                );
                validate_optional_string_prop(component, path, "symbol", errors);
                validate_no_children(component, path, errors);
            }
            "Spacer" => {
                validate_no_children(component, path, errors);
            }
            "Panel" => {
                validate_optional_string_prop(component, path, "title", errors);
            }
            _ => {}
        }
    }
}

fn is_supported_component_kind(kind: &str) -> bool {
    matches!(
        kind,
        "VBox"
            | "HBox"
            | "Panel"
            | "Spacer"
            | "Divider"
            | "Label"
            | "TextBlock"
            | "Button"
            | "List"
            | "Graph"
    )
}

fn validate_no_children(component: &ComponentSpec, path: &str, errors: &mut Vec<ValidationError>) {
    if !component.children.is_empty() {
        errors.push(ValidationError::new(
            format!("{path}.children"),
            format!("{} does not accept child components", component.kind),
        ));
    }
}

fn validate_required_string_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    match component.props.get(prop) {
        Some(Value::String(value)) if !value.trim().is_empty() => {}
        Some(Value::String(_)) => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("property `{prop}` must not be empty"),
        )),
        Some(_) => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("property `{prop}` must be a string"),
        )),
        None => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("missing required property `{prop}`"),
        )),
    }
}

fn validate_optional_string_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    if let Some(value) = component.props.get(prop) {
        match value {
            Value::String(_) => {}
            _ => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("property `{prop}` must be a string"),
            )),
        }
    }
}

fn validate_optional_enum_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    allowed: &[&str],
    errors: &mut Vec<ValidationError>,
) {
    if let Some(value) = component.props.get(prop) {
        match value {
            Value::String(value) if allowed.iter().any(|allowed| value == allowed) => {}
            Value::String(value) => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!(
                    "property `{prop}` must be one of: {} (got `{value}`)",
                    allowed.join(", ")
                ),
            )),
            _ => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("property `{prop}` must be a string"),
            )),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawAppSpec {
    schema_version: Option<String>,
    theme: Option<String>,
    root: Option<RawComponentSpec>,
}

#[derive(Debug, Deserialize)]
struct RawComponentSpec {
    kind: Option<String>,
    id: Option<String>,
    #[serde(default)]
    props: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    children: Vec<RawComponentSpec>,
}

impl TryFrom<RawAppSpec> for AppSpec {
    type Error = DslError;

    fn try_from(value: RawAppSpec) -> Result<Self, Self::Error> {
        Ok(Self {
            schema_version: value.schema_version.ok_or(DslError::MissingSchemaVersion)?,
            theme: value.theme,
            root: value.root.ok_or(DslError::MissingRoot)?.try_into()?,
        })
    }
}

impl TryFrom<RawComponentSpec> for ComponentSpec {
    type Error = DslError;

    fn try_from(value: RawComponentSpec) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: value.kind.ok_or(DslError::MissingComponentKind)?,
            id: value.id,
            props: value
                .props
                .into_iter()
                .map(|(key, value)| (key, Value::from(value)))
                .collect(),
            children: value
                .children
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(value) => Self::Bool(value),
            serde_json::Value::Number(value) => {
                if let Some(value) = value.as_i64() {
                    Self::Integer(value)
                } else {
                    Self::Float(value.to_string())
                }
            }
            serde_json::Value::String(value) => Self::String(value),
            serde_json::Value::Array(values) => {
                Self::Array(values.into_iter().map(Value::from).collect())
            }
            serde_json::Value::Object(values) => Self::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, Value::from(value)))
                    .collect(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_toml_app_spec() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"
theme = "minimal"

[root]
kind = "Label"

[root.props]
text = "Hello NeoTUI"
align = "center"
"#,
        )
        .expect("hello fixture should parse");

        assert_eq!(spec.schema_version, "0.1");
        assert_eq!(spec.theme.as_deref(), Some("minimal"));
        assert_eq!(spec.root.kind, "Label");
        assert_eq!(
            spec.root.props.get("text"),
            Some(&Value::String("Hello NeoTUI".into()))
        );
        assert_eq!(
            spec.root.props.get("align"),
            Some(&Value::String("center".into()))
        );
    }

    #[test]
    fn parses_nested_json_component_tree() {
        let spec = AppSpec::from_json_str(
            r#"{
  "schema_version": "0.1",
  "theme": "dark",
  "root": {
    "kind": "VBox",
    "id": "layout",
    "props": {
      "gap": 1
    },
    "children": [
      {
        "kind": "Label",
        "props": {
          "text": "Headline"
        }
      },
      {
        "kind": "Divider",
        "props": {
          "symbol": "="
        }
      }
    ]
  }
}"#,
        )
        .expect("json spec should parse");

        assert_eq!(spec.theme.as_deref(), Some("dark"));
        assert_eq!(spec.root.id.as_deref(), Some("layout"));
        assert_eq!(spec.root.children.len(), 2);
        assert_eq!(spec.root.props.get("gap"), Some(&Value::Integer(1)));
        assert_eq!(spec.root.children[0].kind, "Label");
        assert_eq!(spec.root.children[1].kind, "Divider");
    }

    #[test]
    fn reports_missing_schema_version() {
        let error = AppSpec::from_json_str(
            r#"{
  "root": {
    "kind": "Label"
  }
}"#,
        )
        .expect_err("missing schema_version should fail");

        assert_eq!(error.to_string(), "app spec must define schema_version");
    }

    #[test]
    fn reports_missing_component_kind() {
        let error = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[root]
id = "missing-kind"
"#,
        )
        .expect_err("missing component kind should fail");

        assert_eq!(error.to_string(), "component spec must define kind");
    }

    #[test]
    fn detects_format_from_path_extension() {
        assert_eq!(
            DslFormat::detect_from_path("examples/hello.toml"),
            Some(DslFormat::Toml)
        );
        assert_eq!(
            DslFormat::detect_from_path("examples/dashboard.JSON"),
            Some(DslFormat::Json)
        );
        assert_eq!(DslFormat::detect_from_path("examples/dashboard.yaml"), None);
    }

    #[test]
    fn preserves_arrays_and_objects_in_props() {
        let spec = AppSpec::from_json_str(
            r#"{
  "schema_version": "0.1",
  "root": {
    "kind": "Graph",
    "props": {
      "values": [1, 2, 3],
      "meta": {
        "title": "Throughput"
      }
    }
  }
}"#,
        )
        .expect("json with nested props should parse");

        assert_eq!(
            spec.root.props.get("values"),
            Some(&Value::Array(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ]))
        );
        assert_eq!(
            spec.root.props.get("meta"),
            Some(&Value::Object(BTreeMap::from([(
                "title".into(),
                Value::String("Throughput".into()),
            )])))
        );
    }

    #[test]
    fn validator_accepts_valid_label_spec() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"
theme = "minimal"

[root]
kind = "Label"

[root.props]
text = "Hello"
align = "center"
"#,
        )
        .expect("valid spec should parse");

        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn validator_reports_unknown_component_and_missing_prop() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            root: ComponentSpec {
                kind: "Unknown".into(),
                id: None,
                props: BTreeMap::new(),
                children: Vec::new(),
            },
        };

        let errors = spec.validate().expect_err("invalid spec should fail");

        assert_eq!(errors.len(), 1);
        assert_eq!(errors.errors()[0].path, "root.kind");
    }

    #[test]
    fn validator_reports_multiple_actionable_errors() {
        let spec = AppSpec {
            schema_version: "".into(),
            theme: Some(" ".into()),
            root: ComponentSpec {
                kind: "Label".into(),
                id: Some("".into()),
                props: BTreeMap::from([
                    ("text".into(), Value::Integer(7)),
                    ("align".into(), Value::String("diagonal".into())),
                ]),
                children: vec![ComponentSpec {
                    kind: "Spacer".into(),
                    id: None,
                    props: BTreeMap::new(),
                    children: Vec::new(),
                }],
            },
        };

        let errors = spec.validate().expect_err("invalid spec should fail");
        let rendered = errors.to_string();

        assert!(rendered.contains("app.schema_version: schema_version must not be empty"));
        assert!(rendered.contains("app.theme: theme must not be empty when provided"));
        assert!(rendered.contains("root.id: component id must not be empty when provided"));
        assert!(rendered.contains("root.props.text: property `text` must be a string"));
        assert!(rendered.contains("root.props.align: property `align` must be one of: left, center, right (got `diagonal`)"));
        assert!(rendered.contains("root.children: Label does not accept child components"));
    }

    #[test]
    fn validator_rejects_invalid_divider_orientation() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            root: ComponentSpec {
                kind: "Divider".into(),
                id: None,
                props: BTreeMap::from([("orientation".into(), Value::String("sideways".into()))]),
                children: Vec::new(),
            },
        };

        let errors = spec.validate().expect_err("invalid spec should fail");

        assert_eq!(errors.errors()[0].path, "root.props.orientation");
    }

    #[test]
    fn parses_dashboard_toml_example() {
        let input = std::fs::read_to_string("examples/dashboard.toml")
            .expect("dashboard example should exist");

        let spec = AppSpec::from_toml_str(&input).expect("dashboard example should parse");

        assert_eq!(spec.theme.as_deref(), Some("dark"));
        assert_eq!(spec.root.kind, "Panel");
        assert_eq!(spec.root.children.len(), 3);
    }

    #[test]
    fn parses_dashboard_json_example() {
        let input =
            std::fs::read_to_string("examples/dashboard.json").expect("json example should exist");

        let spec = AppSpec::from_json_str(&input).expect("json example should parse");

        assert_eq!(spec.theme.as_deref(), Some("minimal"));
        assert_eq!(spec.root.kind, "Panel");
        assert_eq!(spec.root.children.len(), 4);
    }
}
