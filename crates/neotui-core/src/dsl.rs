// DSL model
// Neutral application spec and parsers for canonical TOML/JSON inputs

use std::collections::BTreeMap;
use std::fmt;

use crate::data::{
    ActionKind, ActionSpec, DataBinding, DataSourceSpec, DataSpec, HttpBody, HttpHeaderValue,
    HttpMethod, HttpSourceSpec,
};
use crate::forms::{FormFieldKind, FormFieldSpec, FormSpec};
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSpec {
    pub schema_version: String,
    pub theme: Option<String>,
    pub data: Option<DataSpec>,
    pub actions: Vec<ActionSpec>,
    pub forms: Vec<FormSpec>,
    pub root: ComponentSpec,
}

impl AppSpec {
    pub fn from_toml_str(input: &str) -> Result<Self, DslError> {
        debug!(
            target: "neotui::dsl",
            format = "toml",
            input_bytes = input.len(),
            "parsing app spec"
        );
        let raw: RawAppSpec =
            toml::from_str(input).map_err(|source| DslError::ParseToml { source })?;
        raw.try_into()
    }

    pub fn from_json_str(input: &str) -> Result<Self, DslError> {
        debug!(
            target: "neotui::dsl",
            format = "json",
            input_bytes = input.len(),
            "parsing app spec"
        );
        let raw: RawAppSpec =
            serde_json::from_str(input).map_err(|source| DslError::ParseJson { source })?;
        raw.try_into()
    }

    pub fn validate(&self) -> Result<(), ValidationErrors> {
        debug!(
            target: "neotui::dsl",
            root_kind = self.root.kind.as_str(),
            has_theme = self.theme.is_some(),
            "validating app spec"
        );
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
    InvalidDataSource { message: String },
}

impl fmt::Display for DslError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseToml { source } => write!(f, "failed to parse TOML DSL: {source}"),
            Self::ParseJson { source } => write!(f, "failed to parse JSON DSL: {source}"),
            Self::MissingRoot => write!(f, "app spec must define a root component"),
            Self::MissingSchemaVersion => write!(f, "app spec must define schema_version"),
            Self::MissingComponentKind => write!(f, "component spec must define kind"),
            Self::InvalidDataSource { message } => write!(f, "invalid data source: {message}"),
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

        self.validate_data_sources(spec, &mut errors);
        self.validate_actions(spec, &mut errors);
        self.validate_forms(spec, &mut errors);
        self.validate_component(&spec.root, "root", &mut errors);
        self.validate_action_bindings(spec, &mut errors);
        self.validate_form_bindings(spec, &mut errors);

        if errors.is_empty() {
            debug!(
                target: "neotui::dsl",
                root_kind = spec.root.kind.as_str(),
                "app spec validation passed"
            );
            Ok(())
        } else {
            debug!(
                target: "neotui::dsl",
                root_kind = spec.root.kind.as_str(),
                error_count = errors.len(),
                "app spec validation failed"
            );
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

        validate_optional_non_negative_integer_prop(component, path, "width", errors);
        validate_optional_non_negative_integer_prop(component, path, "height", errors);
        validate_optional_percentage_prop(component, path, "width_pct", errors);
        validate_optional_percentage_prop(component, path, "height_pct", errors);
        validate_optional_positive_integer_prop(component, path, "grow", errors);

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
        validate_bindings(component, path, errors);
        match component.kind.as_str() {
            "Label" => {
                validate_required_string_or_binding_prop(
                    component,
                    path,
                    "text",
                    "text_from",
                    errors,
                );
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
            "VBox" | "HBox" => {
                validate_optional_non_negative_integer_prop(component, path, "gap", errors);
                validate_optional_enum_prop(
                    component,
                    path,
                    "align",
                    &["start", "center", "end", "stretch"],
                    errors,
                );
                validate_optional_enum_prop(
                    component,
                    path,
                    "justify",
                    &["start", "center", "end"],
                    errors,
                );
            }
            "Panel" => {
                validate_optional_string_prop(component, path, "title", errors);
                validate_optional_enum_prop(
                    component,
                    path,
                    "variant",
                    &[
                        "plain", "framed", "data", "alert", "hero", "danger", "warning", "success",
                        "info",
                    ],
                    errors,
                );
                validate_optional_enum_prop(
                    component,
                    path,
                    "density",
                    &["compact", "normal", "spacious"],
                    errors,
                );
                validate_optional_enum_prop(
                    component,
                    path,
                    "chrome",
                    &["minimal", "framed", "technical", "cinematic"],
                    errors,
                );
                validate_optional_enum_prop(
                    component,
                    path,
                    "border_style",
                    &["single", "double", "rounded", "hex", "angular"],
                    errors,
                );
                validate_optional_bool_prop(component, path, "grid", errors);
                validate_optional_bool_prop(component, path, "controls", errors);
                validate_optional_enum_prop(
                    component,
                    path,
                    "title_style",
                    &["plain", "chevron", "bracket", "arrow"],
                    errors,
                );
                validate_optional_string_prop(component, path, "footer_left", errors);
                validate_optional_string_prop(component, path, "footer_right", errors);
            }
            "TextBlock" => {
                validate_required_string_or_binding_prop(
                    component,
                    path,
                    "text",
                    "text_from",
                    errors,
                );
                validate_no_children(component, path, errors);
            }
            "TextInput" => {
                validate_required_string_or_binding_prop(
                    component,
                    path,
                    "value",
                    "value_from",
                    errors,
                );
                validate_required_string_prop(component, path, "form", errors);
                validate_required_string_prop(component, path, "field", errors);
                validate_optional_string_prop(component, path, "placeholder", errors);
                validate_no_children(component, path, errors);
            }
            "Button" => {
                validate_required_string_or_binding_prop(
                    component,
                    path,
                    "text",
                    "text_from",
                    errors,
                );
                validate_optional_string_prop(component, path, "variant", errors);
                validate_optional_string_prop(component, path, "on_click", errors);
                validate_no_children(component, path, errors);
            }
            "List" => {
                validate_required_array_or_binding_prop(
                    component,
                    path,
                    "items",
                    "items_from",
                    errors,
                );
                validate_optional_string_prop(component, path, "title", errors);
                validate_optional_string_prop(component, path, "on_select", errors);
                validate_no_children(component, path, errors);
            }
            "Graph" => {
                validate_required_array_or_binding_prop(
                    component,
                    path,
                    "values",
                    "values_from",
                    errors,
                );
                validate_optional_string_prop(component, path, "title", errors);
                validate_no_children(component, path, errors);
            }
            "Table" => {
                validate_required_table_columns_prop(component, path, errors);
                validate_required_array_or_binding_prop(
                    component,
                    path,
                    "rows",
                    "rows_from",
                    errors,
                );
                validate_no_children(component, path, errors);
            }
            "Metric" => {
                validate_required_string_prop(component, path, "title", errors);
                validate_required_string_or_binding_prop(
                    component,
                    path,
                    "value",
                    "value_from",
                    errors,
                );
                validate_optional_string_prop(component, path, "delta", errors);
                validate_optional_enum_prop(
                    component,
                    path,
                    "status",
                    &[
                        "normal", "loading", "error", "warning", "critical", "info", "success",
                        "danger",
                    ],
                    errors,
                );
                validate_no_children(component, path, errors);
            }
            "Gauge" => {
                validate_required_number_or_binding_prop(
                    component,
                    path,
                    "value",
                    "value_from",
                    errors,
                );
                validate_optional_double_prop(component, path, "min", errors);
                validate_optional_double_prop(component, path, "max", errors);
                validate_optional_enum_prop(
                    component,
                    path,
                    "orientation",
                    &["horizontal", "vertical"],
                    errors,
                );
                validate_optional_string_prop(component, path, "title", errors);
                validate_optional_enum_prop(
                    component,
                    path,
                    "fill_style",
                    &["solid", "gradient", "block"],
                    errors,
                );
                validate_no_children(component, path, errors);
            }
            "Sparkline" => {
                validate_required_array_or_binding_prop(
                    component,
                    path,
                    "values",
                    "values_from",
                    errors,
                );
                validate_optional_string_prop(component, path, "title", errors);
                validate_no_children(component, path, errors);
            }
            "KeyValueRow" => {
                validate_required_string_prop(component, path, "key", errors);
                validate_required_string_or_binding_prop(
                    component,
                    path,
                    "value",
                    "value_from",
                    errors,
                );
                validate_optional_string_prop(component, path, "connector", errors);
                validate_no_children(component, path, errors);
            }
            "StatusStrip" => {
                validate_required_string_or_binding_prop(
                    component,
                    path,
                    "text",
                    "text_from",
                    errors,
                );
                validate_optional_enum_prop(
                    component,
                    path,
                    "status",
                    &[
                        "normal", "loading", "error", "warning", "critical", "info", "success",
                        "danger",
                    ],
                    errors,
                );
                validate_optional_enum_prop(
                    component,
                    path,
                    "fill",
                    &["chevron", "arrow", "dots"],
                    errors,
                );
                validate_no_children(component, path, errors);
            }
            "BigMetric" => {
                validate_required_string_or_binding_prop(
                    component,
                    path,
                    "value",
                    "value_from",
                    errors,
                );
                validate_optional_string_prop(component, path, "title", errors);
                validate_optional_string_prop(component, path, "unit", errors);
                validate_optional_enum_prop(
                    component,
                    path,
                    "font",
                    &["compact", "large", "hero"],
                    errors,
                );
                // Legacy: scale=1/2/3 maps to compact/large/hero
                validate_optional_non_negative_integer_prop(component, path, "scale", errors);
                validate_no_children(component, path, errors);
            }
            "Knob" => {
                validate_required_double_prop(component, path, "value", errors);
                validate_optional_double_prop(component, path, "min", errors);
                validate_optional_double_prop(component, path, "max", errors);
                validate_optional_string_prop(component, path, "title", errors);
                validate_no_children(component, path, errors);
            }
            _ => {}
        }
    }

    fn validate_data_sources(&self, spec: &AppSpec, errors: &mut Vec<ValidationError>) {
        let Some(data) = &spec.data else {
            return;
        };
        let mut ids = std::collections::HashSet::new();

        for (index, source) in data.sources.iter().enumerate() {
            let path = format!("data.sources[{index}]");
            match source {
                DataSourceSpec::Http(source) => {
                    if source.id.trim().is_empty() {
                        errors.push(ValidationError::new(
                            format!("{path}.id"),
                            "data source id must not be empty",
                        ));
                    }
                    if !ids.insert(source.id.clone()) {
                        errors.push(ValidationError::new(
                            format!("{path}.id"),
                            format!("duplicate data source id `{}`", source.id),
                        ));
                    }
                    if source.url.trim().is_empty() {
                        errors.push(ValidationError::new(
                            format!("{path}.url"),
                            "HTTP data source url must not be empty",
                        ));
                    }
                    if matches!(source.timeout_ms, Some(0)) {
                        errors.push(ValidationError::new(
                            format!("{path}.timeout_ms"),
                            "timeout_ms must be greater than zero when provided",
                        ));
                    }
                    if matches!(source.refresh_ms, Some(0)) {
                        errors.push(ValidationError::new(
                            format!("{path}.refresh_ms"),
                            "refresh_ms must be greater than zero when provided",
                        ));
                    }
                    for (name, value) in &source.headers {
                        if name.trim().is_empty() {
                            errors.push(ValidationError::new(
                                format!("{path}.headers"),
                                "header names must not be empty",
                            ));
                        }
                        if let HttpHeaderValue::Env { env, .. } = value {
                            if env.trim().is_empty() {
                                errors.push(ValidationError::new(
                                    format!("{path}.headers.{name}.env"),
                                    "env header reference must not be empty",
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    fn validate_actions(&self, spec: &AppSpec, errors: &mut Vec<ValidationError>) {
        let mut ids = std::collections::HashSet::new();
        let data_source_ids = spec
            .data
            .as_ref()
            .map(|data| {
                data.sources
                    .iter()
                    .map(|source| source.id().to_string())
                    .collect::<std::collections::HashSet<_>>()
            })
            .unwrap_or_default();

        for (index, action) in spec.actions.iter().enumerate() {
            let path = format!("actions[{index}]");
            if action.id.trim().is_empty() {
                errors.push(ValidationError::new(
                    format!("{path}.id"),
                    "action id must not be empty",
                ));
            }
            if !ids.insert(action.id.clone()) {
                errors.push(ValidationError::new(
                    format!("{path}.id"),
                    format!("duplicate action id `{}`", action.id),
                ));
            }
            if action.http.url.trim().is_empty() {
                errors.push(ValidationError::new(
                    format!("{path}.url"),
                    "HTTP action url must not be empty",
                ));
            }
            if matches!(action.http.timeout_ms, Some(0)) {
                errors.push(ValidationError::new(
                    format!("{path}.timeout_ms"),
                    "timeout_ms must be greater than zero when provided",
                ));
            }
            for (name, value) in &action.http.headers {
                if name.trim().is_empty() {
                    errors.push(ValidationError::new(
                        format!("{path}.headers"),
                        "header names must not be empty",
                    ));
                }
                if let HttpHeaderValue::Env { env, .. } = value {
                    if env.trim().is_empty() {
                        errors.push(ValidationError::new(
                            format!("{path}.headers.{name}.env"),
                            "env header reference must not be empty",
                        ));
                    }
                }
            }
            for source_id in &action.refresh_sources {
                if !data_source_ids.contains(source_id) {
                    errors.push(ValidationError::new(
                        format!("{path}.refresh_sources"),
                        format!("unknown data source `{source_id}`"),
                    ));
                }
            }
        }
    }

    fn validate_forms(&self, spec: &AppSpec, errors: &mut Vec<ValidationError>) {
        let mut form_ids = std::collections::HashSet::new();

        for (form_index, form) in spec.forms.iter().enumerate() {
            let path = format!("forms[{form_index}]");
            if form.id.trim().is_empty() {
                errors.push(ValidationError::new(
                    format!("{path}.id"),
                    "form id must not be empty",
                ));
            }
            if !form_ids.insert(form.id.as_str()) {
                errors.push(ValidationError::new(
                    format!("{path}.id"),
                    format!("duplicate form id `{}`", form.id),
                ));
            }

            let mut field_ids = std::collections::HashSet::new();
            for (field_index, field) in form.fields.iter().enumerate() {
                let field_path = format!("{path}.fields[{field_index}]");
                if field.id.trim().is_empty() {
                    errors.push(ValidationError::new(
                        format!("{field_path}.id"),
                        "form field id must not be empty",
                    ));
                }
                if !field_ids.insert(field.id.as_str()) {
                    errors.push(ValidationError::new(
                        format!("{field_path}.id"),
                        format!("duplicate form field id `{}`", field.id),
                    ));
                }
            }
        }
    }

    fn validate_action_bindings(&self, spec: &AppSpec, errors: &mut Vec<ValidationError>) {
        let action_ids = spec
            .actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        validate_component_action_bindings(&spec.root, "root", &action_ids, errors);
    }

    fn validate_form_bindings(&self, spec: &AppSpec, errors: &mut Vec<ValidationError>) {
        let forms = spec
            .forms
            .iter()
            .map(|form| {
                (
                    form.id.as_str(),
                    form.fields
                        .iter()
                        .map(|field| field.id.as_str())
                        .collect::<std::collections::HashSet<_>>(),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();
        validate_component_form_bindings(&spec.root, "root", &forms, errors);
        validate_component_form_targets(&spec.root, "root", &forms, errors);
        validate_action_form_templates(spec, &forms, errors);
    }
}

fn validate_action_form_templates(
    spec: &AppSpec,
    forms: &std::collections::HashMap<&str, std::collections::HashSet<&str>>,
    errors: &mut Vec<ValidationError>,
) {
    for (index, action) in spec.actions.iter().enumerate() {
        let Some(body) = &action.http.body else {
            continue;
        };
        validate_form_templates_in_body(body, &format!("actions[{index}].body"), forms, errors);
    }
}

fn validate_form_templates_in_body(
    body: &HttpBody,
    path: &str,
    forms: &std::collections::HashMap<&str, std::collections::HashSet<&str>>,
    errors: &mut Vec<ValidationError>,
) {
    match body {
        HttpBody::Text(text) => validate_form_template_text(text, path, forms, errors),
        HttpBody::Json(value) => validate_form_templates_in_value(value, path, forms, errors),
    }
}

fn validate_form_templates_in_value(
    value: &Value,
    path: &str,
    forms: &std::collections::HashMap<&str, std::collections::HashSet<&str>>,
    errors: &mut Vec<ValidationError>,
) {
    match value {
        Value::String(text) => validate_form_template_text(text, path, forms, errors),
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                validate_form_templates_in_value(value, &format!("{path}[{index}]"), forms, errors);
            }
        }
        Value::Object(values) => {
            for (key, value) in values {
                validate_form_templates_in_value(value, &format!("{path}.{key}"), forms, errors);
            }
        }
        _ => {}
    }
}

fn validate_form_template_text(
    text: &str,
    path: &str,
    forms: &std::collections::HashMap<&str, std::collections::HashSet<&str>>,
    errors: &mut Vec<ValidationError>,
) {
    if !text.starts_with("$forms.") {
        return;
    }
    let Ok(binding) = DataBinding::parse(text) else {
        return;
    };
    let Some(form_id) = binding.path.first() else {
        errors.push(ValidationError::new(
            path,
            "form template must include a form id",
        ));
        return;
    };
    let Some(fields) = forms.get(form_id.as_str()) else {
        errors.push(ValidationError::new(
            path,
            format!("unknown form `{form_id}`"),
        ));
        return;
    };
    let Some(field_id) = binding.path.get(1) else {
        errors.push(ValidationError::new(
            path,
            "form template must include a field id",
        ));
        return;
    };
    if !fields.contains(field_id.as_str()) {
        errors.push(ValidationError::new(
            path,
            format!("unknown form field `{field_id}`"),
        ));
    }
    if binding.path.len() > 2 {
        errors.push(ValidationError::new(
            path,
            "form template must use `$forms.<form_id>.<field_id>`",
        ));
    }
}

fn validate_component_action_bindings(
    component: &ComponentSpec,
    path: &str,
    action_ids: &std::collections::HashSet<&str>,
    errors: &mut Vec<ValidationError>,
) {
    for prop in ["on_click", "on_select"] {
        let Some(Value::String(action_id)) = component.props.get(prop) else {
            continue;
        };
        if !action_ids.contains(action_id.as_str()) {
            errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("unknown action `{action_id}`"),
            ));
        }
    }
    for prop in [
        "text_from",
        "value_from",
        "items_from",
        "values_from",
        "rows_from",
        "status_from",
    ] {
        let Some(Value::String(binding_text)) = component.props.get(prop) else {
            continue;
        };
        let Ok(binding) = DataBinding::parse(binding_text) else {
            continue;
        };
        if binding.source != "$actions" {
            continue;
        }
        let Some(action_id) = binding.path.first() else {
            errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                "action binding must include an action id",
            ));
            continue;
        };
        if !action_ids.contains(action_id.as_str()) {
            errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("unknown action `{action_id}`"),
            ));
        }
        match binding.path.get(1).map(String::as_str) {
            Some("$status" | "error") => {}
            Some(field) => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("unknown action binding field `{field}`"),
            )),
            None => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                "action binding must include `$status` or `error`",
            )),
        }
    }

    for (index, child) in component.children.iter().enumerate() {
        validate_component_action_bindings(
            child,
            &format!("{path}.children[{index}]"),
            action_ids,
            errors,
        );
    }
}

fn validate_component_form_bindings(
    component: &ComponentSpec,
    path: &str,
    forms: &std::collections::HashMap<&str, std::collections::HashSet<&str>>,
    errors: &mut Vec<ValidationError>,
) {
    for prop in [
        "text_from",
        "value_from",
        "items_from",
        "values_from",
        "rows_from",
        "status_from",
    ] {
        let Some(Value::String(binding_text)) = component.props.get(prop) else {
            continue;
        };
        let Ok(binding) = DataBinding::parse(binding_text) else {
            continue;
        };
        if binding.source != "$forms" {
            continue;
        }
        let Some(form_id) = binding.path.first() else {
            errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                "form binding must include a form id",
            ));
            continue;
        };
        let Some(fields) = forms.get(form_id.as_str()) else {
            errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("unknown form `{form_id}`"),
            ));
            continue;
        };
        let Some(field_id) = binding.path.get(1) else {
            errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                "form binding must include a field id",
            ));
            continue;
        };
        if !fields.contains(field_id.as_str()) {
            errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("unknown form field `{field_id}`"),
            ));
        }
        if binding.path.len() > 2 {
            errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                "form binding must use `$forms.<form_id>.<field_id>`",
            ));
        }
    }

    for (index, child) in component.children.iter().enumerate() {
        validate_component_form_bindings(
            child,
            &format!("{path}.children[{index}]"),
            forms,
            errors,
        );
    }
}

fn validate_component_form_targets(
    component: &ComponentSpec,
    path: &str,
    forms: &std::collections::HashMap<&str, std::collections::HashSet<&str>>,
    errors: &mut Vec<ValidationError>,
) {
    if component.kind == "TextInput" {
        let form = component.props.get("form").and_then(|value| match value {
            Value::String(value) => Some(value.as_str()),
            _ => None,
        });
        let field = component.props.get("field").and_then(|value| match value {
            Value::String(value) => Some(value.as_str()),
            _ => None,
        });

        if let Some(form_id) = form {
            match forms.get(form_id) {
                Some(fields) => {
                    if let Some(field_id) = field {
                        if !fields.contains(field_id) {
                            errors.push(ValidationError::new(
                                format!("{path}.props.field"),
                                format!("unknown form field `{field_id}`"),
                            ));
                        }
                    }
                }
                None => errors.push(ValidationError::new(
                    format!("{path}.props.form"),
                    format!("unknown form `{form_id}`"),
                )),
            }
        }
    }

    for (index, child) in component.children.iter().enumerate() {
        validate_component_form_targets(child, &format!("{path}.children[{index}]"), forms, errors);
    }
}

fn validate_bindings(component: &ComponentSpec, path: &str, errors: &mut Vec<ValidationError>) {
    for prop in [
        "text_from",
        "value_from",
        "items_from",
        "values_from",
        "rows_from",
        "status_from",
    ] {
        if let Some(value) = component.props.get(prop) {
            match value {
                Value::String(binding) => {
                    if let Err(message) = DataBinding::parse(binding) {
                        errors.push(ValidationError::new(
                            format!("{path}.props.{prop}"),
                            message,
                        ));
                    }
                }
                _ => errors.push(ValidationError::new(
                    format!("{path}.props.{prop}"),
                    format!("property `{prop}` must be a string data binding"),
                )),
            }
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
            | "TextInput"
            | "Button"
            | "List"
            | "Table"
            | "Graph"
            | "Metric"
            | "Gauge"
            | "Sparkline"
            | "KeyValueRow"
            | "StatusStrip"
            | "Knob"
            | "BigMetric"
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

fn validate_required_string_or_binding_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    binding_prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    if component.props.contains_key(binding_prop) {
        validate_optional_string_prop(component, path, prop, errors);
    } else {
        validate_required_string_prop(component, path, prop, errors);
    }
}

fn validate_required_number_or_binding_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    binding_prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    if component.props.contains_key(binding_prop) {
        validate_optional_double_prop(component, path, prop, errors);
    } else {
        validate_required_double_prop(component, path, prop, errors);
    }
}

fn validate_required_array_or_binding_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    binding_prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    if component.props.contains_key(binding_prop) {
        match component.props.get(prop) {
            Some(Value::Array(_)) | None => {}
            Some(_) => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("property `{prop}` must be an array when provided"),
            )),
        }
    } else {
        match prop {
            "items" => validate_required_string_array_prop(component, path, prop, errors),
            "values" => validate_required_number_array_prop(component, path, prop, errors),
            "rows" => validate_required_object_array_prop(component, path, prop, errors),
            _ => {}
        }
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

fn validate_required_string_array_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    match component.props.get(prop) {
        Some(Value::Array(values))
            if values
                .iter()
                .all(|value| matches!(value, Value::String(item) if !item.trim().is_empty())) => {}
        Some(Value::Array(_)) => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("property `{prop}` must be an array of non-empty strings"),
        )),
        Some(_) => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("property `{prop}` must be an array of non-empty strings"),
        )),
        None => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("missing required property `{prop}`"),
        )),
    }
}

fn validate_required_number_array_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    match component.props.get(prop) {
        Some(Value::Array(values))
            if values
                .iter()
                .all(|value| matches!(value, Value::Integer(_) | Value::Float(_))) => {}
        Some(Value::Array(_)) => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("property `{prop}` must be an array of numbers"),
        )),
        Some(_) => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("property `{prop}` must be an array of numbers"),
        )),
        None => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("missing required property `{prop}`"),
        )),
    }
}

fn validate_required_object_array_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    match component.props.get(prop) {
        Some(Value::Array(values))
            if values.iter().all(|value| matches!(value, Value::Object(_))) => {}
        Some(Value::Array(_)) => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("property `{prop}` must be an array of objects"),
        )),
        Some(_) => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("property `{prop}` must be an array of objects"),
        )),
        None => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("missing required property `{prop}`"),
        )),
    }
}

fn validate_required_table_columns_prop(
    component: &ComponentSpec,
    path: &str,
    errors: &mut Vec<ValidationError>,
) {
    let Some(value) = component.props.get("columns") else {
        errors.push(ValidationError::new(
            format!("{path}.props.columns"),
            "missing required property `columns`",
        ));
        return;
    };

    let Value::Array(columns) = value else {
        errors.push(ValidationError::new(
            format!("{path}.props.columns"),
            "property `columns` must be an array of column objects",
        ));
        return;
    };

    if columns.is_empty() {
        errors.push(ValidationError::new(
            format!("{path}.props.columns"),
            "property `columns` must contain at least one column",
        ));
        return;
    }

    for (index, column) in columns.iter().enumerate() {
        let column_path = format!("{path}.props.columns[{index}]");
        let Value::Object(column) = column else {
            errors.push(ValidationError::new(
                column_path,
                "column must be an object",
            ));
            continue;
        };

        validate_object_required_string_prop(column, &column_path, "key", errors);
        validate_object_required_string_prop(column, &column_path, "title", errors);
        validate_object_required_positive_integer_prop(column, &column_path, "width", errors);

        if let Some(value) = column.get("align") {
            match value {
                Value::String(value) if matches!(value.as_str(), "left" | "center" | "right") => {}
                Value::String(value) => errors.push(ValidationError::new(
                    format!("{column_path}.align"),
                    format!("property `align` must be one of: left, center, right (got `{value}`)"),
                )),
                _ => errors.push(ValidationError::new(
                    format!("{column_path}.align"),
                    "property `align` must be a string",
                )),
            }
        }
    }
}

fn validate_object_required_string_prop(
    object: &BTreeMap<String, Value>,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    match object.get(prop) {
        Some(Value::String(value)) if !value.trim().is_empty() => {}
        Some(Value::String(_)) => errors.push(ValidationError::new(
            format!("{path}.{prop}"),
            format!("property `{prop}` must not be empty"),
        )),
        Some(_) => errors.push(ValidationError::new(
            format!("{path}.{prop}"),
            format!("property `{prop}` must be a string"),
        )),
        None => errors.push(ValidationError::new(
            format!("{path}.{prop}"),
            format!("missing required property `{prop}`"),
        )),
    }
}

fn validate_object_required_positive_integer_prop(
    object: &BTreeMap<String, Value>,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    match object.get(prop) {
        Some(Value::Integer(value)) if *value > 0 => {}
        Some(Value::Integer(_)) => errors.push(ValidationError::new(
            format!("{path}.{prop}"),
            format!("property `{prop}` must be greater than zero"),
        )),
        Some(_) => errors.push(ValidationError::new(
            format!("{path}.{prop}"),
            format!("property `{prop}` must be greater than zero"),
        )),
        None => errors.push(ValidationError::new(
            format!("{path}.{prop}"),
            format!("missing required property `{prop}`"),
        )),
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

fn validate_optional_non_negative_integer_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    if let Some(value) = component.props.get(prop) {
        match value {
            Value::Integer(value) if *value >= 0 => {}
            Value::Integer(_) => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("property `{prop}` must be a non-negative integer"),
            )),
            _ => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("property `{prop}` must be a non-negative integer"),
            )),
        }
    }
}

fn validate_optional_positive_integer_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    if let Some(value) = component.props.get(prop) {
        match value {
            Value::Integer(value) if *value > 0 => {}
            Value::Integer(_) => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("property `{prop}` must be greater than zero"),
            )),
            _ => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("property `{prop}` must be greater than zero"),
            )),
        }
    }
}

fn validate_optional_percentage_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    if let Some(value) = component.props.get(prop) {
        match value {
            Value::Integer(value) if (0..=100).contains(value) => {}
            Value::Integer(_) => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("property `{prop}` must be between 0 and 100"),
            )),
            _ => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("property `{prop}` must be between 0 and 100"),
            )),
        }
    }
}

fn validate_required_double_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    match component.props.get(prop) {
        Some(Value::Integer(_)) | Some(Value::Float(_)) => {}
        Some(_) => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("property `{prop}` must be a number"),
        )),
        None => errors.push(ValidationError::new(
            format!("{path}.props.{prop}"),
            format!("missing required property `{prop}`"),
        )),
    }
}

fn validate_optional_double_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    if let Some(value) = component.props.get(prop) {
        match value {
            Value::Integer(_) | Value::Float(_) => {}
            _ => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("property `{prop}` must be a number"),
            )),
        }
    }
}

fn validate_optional_bool_prop(
    component: &ComponentSpec,
    path: &str,
    prop: &str,
    errors: &mut Vec<ValidationError>,
) {
    if let Some(value) = component.props.get(prop) {
        match value {
            Value::Bool(_) => {}
            _ => errors.push(ValidationError::new(
                format!("{path}.props.{prop}"),
                format!("property `{prop}` must be a boolean"),
            )),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawAppSpec {
    schema_version: Option<String>,
    theme: Option<String>,
    data: Option<RawDataSpec>,
    #[serde(default)]
    actions: Vec<RawActionSpec>,
    #[serde(default)]
    forms: Vec<RawFormSpec>,
    root: Option<RawComponentSpec>,
}

#[derive(Debug, Deserialize)]
struct RawDataSpec {
    #[serde(default)]
    sources: Vec<RawDataSourceSpec>,
}

#[derive(Debug, Deserialize)]
struct RawDataSourceSpec {
    id: Option<String>,
    kind: Option<String>,
    url: Option<String>,
    method: Option<String>,
    #[serde(default)]
    headers: BTreeMap<String, serde_json::Value>,
    body: Option<serde_json::Value>,
    timeout_ms: Option<u64>,
    refresh_ms: Option<u64>,
    retry_count: Option<u16>,
    retry_backoff_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct RawActionSpec {
    id: Option<String>,
    kind: Option<String>,
    url: Option<String>,
    method: Option<String>,
    #[serde(default)]
    headers: BTreeMap<String, serde_json::Value>,
    body: Option<serde_json::Value>,
    timeout_ms: Option<u64>,
    retry_count: Option<u16>,
    retry_backoff_ms: Option<u64>,
    #[serde(default)]
    refresh_sources: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawFormSpec {
    id: Option<String>,
    #[serde(default)]
    fields: Vec<RawFormFieldSpec>,
}

#[derive(Debug, Deserialize)]
struct RawFormFieldSpec {
    id: Option<String>,
    kind: Option<String>,
    initial: Option<serde_json::Value>,
    required: Option<bool>,
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
            data: value.data.map(TryInto::try_into).transpose()?,
            actions: value
                .actions
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            forms: value
                .forms
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            root: value.root.ok_or(DslError::MissingRoot)?.try_into()?,
        })
    }
}

impl TryFrom<RawFormSpec> for FormSpec {
    type Error = DslError;

    fn try_from(value: RawFormSpec) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.ok_or_else(|| DslError::InvalidDataSource {
                message: "form must define id".into(),
            })?,
            fields: value
                .fields
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<RawFormFieldSpec> for FormFieldSpec {
    type Error = DslError;

    fn try_from(value: RawFormFieldSpec) -> Result<Self, Self::Error> {
        let kind = value
            .kind
            .as_deref()
            .unwrap_or("text")
            .parse::<FormFieldKind>()
            .map_err(|message| DslError::InvalidDataSource { message })?;
        Ok(Self {
            id: value.id.ok_or_else(|| DslError::InvalidDataSource {
                message: "form field must define id".into(),
            })?,
            kind,
            initial: value.initial.map(Value::from),
            required: value.required.unwrap_or(false),
        })
    }
}

impl TryFrom<RawActionSpec> for ActionSpec {
    type Error = DslError;

    fn try_from(value: RawActionSpec) -> Result<Self, Self::Error> {
        let kind = value.kind.unwrap_or_else(|| "http".into());
        if kind != "http" {
            return Err(DslError::InvalidDataSource {
                message: format!("unsupported action kind `{kind}`"),
            });
        }
        let id = value.id.ok_or_else(|| DslError::InvalidDataSource {
            message: "HTTP action must define id".into(),
        })?;
        let method = value
            .method
            .as_deref()
            .unwrap_or("POST")
            .parse::<HttpMethod>()
            .map_err(|message| DslError::InvalidDataSource { message })?;
        let http = HttpSourceSpec {
            id: id.clone(),
            url: value.url.ok_or_else(|| DslError::InvalidDataSource {
                message: "HTTP action must define url".into(),
            })?,
            method,
            headers: value
                .headers
                .into_iter()
                .map(|(name, value)| parse_header_value(name, value))
                .collect::<Result<BTreeMap<_, _>, _>>()?,
            body: value.body.map(parse_http_body).transpose()?,
            timeout_ms: value.timeout_ms,
            refresh_ms: None,
            retry_count: value.retry_count.unwrap_or(0),
            retry_backoff_ms: value.retry_backoff_ms.unwrap_or(0),
        };

        Ok(Self {
            id,
            kind: ActionKind::Http,
            http,
            refresh_sources: value.refresh_sources,
        })
    }
}

impl TryFrom<RawDataSpec> for DataSpec {
    type Error = DslError;

    fn try_from(value: RawDataSpec) -> Result<Self, Self::Error> {
        Ok(Self {
            sources: value
                .sources
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<RawDataSourceSpec> for DataSourceSpec {
    type Error = DslError;

    fn try_from(value: RawDataSourceSpec) -> Result<Self, Self::Error> {
        let kind = value.kind.unwrap_or_else(|| "http".into());
        if kind != "http" {
            return Err(DslError::InvalidDataSource {
                message: format!("unsupported data source kind `{kind}`"),
            });
        }

        let method = value
            .method
            .as_deref()
            .unwrap_or("GET")
            .parse::<HttpMethod>()
            .map_err(|message| DslError::InvalidDataSource { message })?;

        Ok(Self::Http(HttpSourceSpec {
            id: value.id.ok_or_else(|| DslError::InvalidDataSource {
                message: "HTTP data source must define id".into(),
            })?,
            url: value.url.ok_or_else(|| DslError::InvalidDataSource {
                message: "HTTP data source must define url".into(),
            })?,
            method,
            headers: value
                .headers
                .into_iter()
                .map(|(name, value)| parse_header_value(name, value))
                .collect::<Result<BTreeMap<_, _>, _>>()?,
            body: value.body.map(parse_http_body).transpose()?,
            timeout_ms: value.timeout_ms,
            refresh_ms: value.refresh_ms,
            retry_count: value.retry_count.unwrap_or(0),
            retry_backoff_ms: value.retry_backoff_ms.unwrap_or(0),
        }))
    }
}

fn parse_header_value(
    name: String,
    value: serde_json::Value,
) -> Result<(String, HttpHeaderValue), DslError> {
    match value {
        serde_json::Value::String(value) => Ok((name, HttpHeaderValue::Literal(value))),
        serde_json::Value::Object(mut object) => {
            let secret = object
                .remove("secret")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            if let Some(env_value) = object.remove("env") {
                let Some(env) = env_value.as_str() else {
                    return Err(DslError::InvalidDataSource {
                        message: format!("header `{name}` env reference must be a string"),
                    });
                };
                let prefix = optional_json_string(object.remove("prefix"), "prefix")?;
                let suffix = optional_json_string(object.remove("suffix"), "suffix")?;
                return Ok((
                    name,
                    HttpHeaderValue::Env {
                        env: env.into(),
                        prefix,
                        suffix,
                    },
                ));
            }
            if let Some(literal_value) = object.remove("value") {
                if secret {
                    return Err(DslError::InvalidDataSource {
                        message: format!(
                            "header `{name}` is marked secret and must use an env reference"
                        ),
                    });
                }
                let Some(literal) = literal_value.as_str() else {
                    return Err(DslError::InvalidDataSource {
                        message: format!("header `{name}` value must be a string"),
                    });
                };
                return Ok((name, HttpHeaderValue::Literal(literal.into())));
            }
            Err(DslError::InvalidDataSource {
                message: format!("header `{name}` must define `env` or `value`"),
            })
        }
        _ => Err(DslError::InvalidDataSource {
            message: format!("header `{name}` must be a string or object"),
        }),
    }
}

fn optional_json_string(
    value: Option<serde_json::Value>,
    property: &str,
) -> Result<Option<String>, DslError> {
    match value {
        Some(serde_json::Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(DslError::InvalidDataSource {
            message: format!("header `{property}` must be a string"),
        }),
        None => Ok(None),
    }
}

fn parse_http_body(value: serde_json::Value) -> Result<HttpBody, DslError> {
    match value {
        serde_json::Value::String(value) => Ok(HttpBody::Text(value)),
        serde_json::Value::Object(mut object) => {
            if let Some(text) = object.remove("text") {
                let Some(text) = text.as_str() else {
                    return Err(DslError::InvalidDataSource {
                        message: "HTTP body.text must be a string".into(),
                    });
                };
                return Ok(HttpBody::Text(text.into()));
            }
            if let Some(json) = object.remove("json") {
                return Ok(HttpBody::Json(Value::from(json)));
            }
            Ok(HttpBody::Json(Value::from(serde_json::Value::Object(
                object,
            ))))
        }
        other => Ok(HttpBody::Json(Value::from(other))),
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
    use std::path::PathBuf;

    fn fixture_path(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }

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
    fn parses_http_data_source_toml() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[[data.sources]]
id = "ops"
kind = "http"
url = "http://127.0.0.1:7878/status"
method = "POST"
timeout_ms = 1000
refresh_ms = 5000

[data.sources.headers.Authorization]
env = "NEOTUI_API_TOKEN"
prefix = "Bearer "

[data.sources.body]
json = { ping = true }

[root]
kind = "Label"

[root.props]
text_from = "ops.summary"
"#,
        )
        .expect("HTTP data source should parse");

        let data = spec.data.expect("data spec should exist");
        assert_eq!(data.sources.len(), 1);
        assert_eq!(data.sources[0].id(), "ops");
    }

    #[test]
    fn parses_form_state_toml() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[[forms]]
id = "incident"

[[forms.fields]]
id = "summary"
kind = "text"
initial = "Disk full"
required = true

[root]
kind = "Label"

[root.props]
text_from = "$forms.incident.summary"
"#,
        )
        .expect("form spec should parse");

        assert_eq!(spec.forms.len(), 1);
        assert_eq!(spec.forms[0].id, "incident");
        assert_eq!(spec.forms[0].fields[0].id, "summary");
        assert_eq!(
            spec.forms[0].fields[0].initial,
            Some(Value::String("Disk full".into()))
        );
        spec.validate().expect("form binding should validate");
    }

    #[test]
    fn parses_http_action_toml() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[[data.sources]]
id = "ops"
kind = "http"
url = "http://127.0.0.1:7878/status"

[[actions]]
id = "refresh_now"
kind = "http"
url = "http://127.0.0.1:7878/ack"
method = "POST"
refresh_sources = ["ops"]

[actions.body]
json = { intent = "refresh" }

[root]
kind = "Button"

[root.props]
text = "Refresh"
on_click = "refresh_now"
"#,
        )
        .expect("HTTP action should parse");

        assert_eq!(spec.actions.len(), 1);
        assert_eq!(spec.actions[0].id, "refresh_now");
        assert_eq!(spec.actions[0].refresh_sources, vec!["ops"]);
        spec.validate().expect("action fixture should validate");
    }

    #[test]
    fn validator_rejects_action_refreshing_unknown_source() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[[actions]]
id = "refresh_now"
kind = "http"
url = "http://127.0.0.1:7878/ack"
refresh_sources = ["missing"]

[root]
kind = "Button"

[root.props]
text = "Refresh"
on_click = "refresh_now"
"#,
        )
        .expect("HTTP action should parse");

        let errors = spec
            .validate()
            .expect_err("unknown refresh source should fail");
        assert!(errors.to_string().contains("unknown data source `missing`"));
    }

    #[test]
    fn validator_rejects_unknown_widget_action_binding() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[root]
kind = "Button"

[root.props]
text = "Refresh"
on_click = "missing_action"
"#,
        )
        .expect("button action binding should parse");

        let errors = spec
            .validate()
            .expect_err("unknown widget action should fail");
        assert!(errors
            .to_string()
            .contains("unknown action `missing_action`"));
    }

    #[test]
    fn validator_rejects_invalid_action_timeout() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[[actions]]
id = "refresh_now"
kind = "http"
url = "http://127.0.0.1:7878/ack"
timeout_ms = 0

[root]
kind = "Button"

[root.props]
text = "Refresh"
on_click = "refresh_now"
"#,
        )
        .expect("HTTP action should parse");

        let errors = spec
            .validate()
            .expect_err("zero action timeout should fail");
        assert!(errors
            .to_string()
            .contains("timeout_ms must be greater than zero"));
    }

    #[test]
    fn validator_rejects_unknown_action_status_binding() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[root]
kind = "StatusStrip"

[root.props]
text = "Action"
status = "info"
status_from = "$actions.missing_action.$status"
"#,
        )
        .expect("action status binding should parse");

        let errors = spec
            .validate()
            .expect_err("unknown action status binding should fail");
        assert!(errors
            .to_string()
            .contains("unknown action `missing_action`"));
    }

    #[test]
    fn validator_rejects_unknown_action_binding_field() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[[actions]]
id = "refresh_now"
kind = "http"
url = "http://127.0.0.1:7878/ack"

[root]
kind = "StatusStrip"

[root.props]
text = "Action"
status = "info"
status_from = "$actions.refresh_now.result"
"#,
        )
        .expect("action status binding should parse");

        let errors = spec
            .validate()
            .expect_err("unknown action binding field should fail");
        assert!(errors
            .to_string()
            .contains("unknown action binding field `result`"));
    }

    #[test]
    fn validator_rejects_unknown_form_binding() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[[forms]]
id = "incident"

[[forms.fields]]
id = "summary"

[root]
kind = "Label"

[root.props]
text_from = "$forms.incident.details"
"#,
        )
        .expect("form binding spec should parse");

        let errors = spec.validate().expect_err("unknown form field should fail");

        assert!(errors.to_string().contains("unknown form field `details`"));
    }

    #[test]
    fn validator_rejects_duplicate_form_fields() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[[forms]]
id = "incident"

[[forms.fields]]
id = "summary"

[[forms.fields]]
id = "summary"

[root]
kind = "Label"

[root.props]
text = "Form"
"#,
        )
        .expect("duplicate field spec should parse");

        let errors = spec
            .validate()
            .expect_err("duplicate form fields should fail");

        assert!(errors
            .to_string()
            .contains("duplicate form field id `summary`"));
    }

    #[test]
    fn validator_rejects_unknown_form_action_body_template() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[[forms]]
id = "incident"

[[forms.fields]]
id = "summary"

[[actions]]
id = "submit_incident"
kind = "http"
url = "http://127.0.0.1:7878/ack"

[actions.body]
json = { summary = "$forms.incident.details" }

[root]
kind = "Button"

[root.props]
text = "Submit"
on_click = "submit_incident"
"#,
        )
        .expect("form action body spec should parse");

        let errors = spec
            .validate()
            .expect_err("unknown form body template should fail");

        assert!(errors.to_string().contains("unknown form field `details`"));
    }

    #[test]
    fn rejects_secret_literal_headers() {
        let error = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[[data.sources]]
id = "ops"
kind = "http"
url = "http://127.0.0.1:7878/status"

[data.sources.headers.Authorization]
value = "Bearer secret"
secret = true

[root]
kind = "Label"

[root.props]
text = "safe"
"#,
        )
        .expect_err("secret literal headers should be rejected");

        assert!(error.to_string().contains("must use an env reference"));
        assert!(!error.to_string().contains("Bearer secret"));
    }

    #[test]
    fn validates_binding_without_static_required_prop() {
        let spec = AppSpec::from_toml_str(
            r#"
schema_version = "0.1"

[[data.sources]]
id = "ops"
kind = "http"
url = "http://127.0.0.1:7878/status"

[root]
kind = "Label"

[root.props]
text_from = "ops.summary"
"#,
        )
        .expect("binding fixture should parse");

        spec.validate()
            .expect("binding should satisfy required text prop");
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
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
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
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
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
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
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
    fn validator_accepts_vbox_gap_prop() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "VBox".into(),
                id: None,
                props: BTreeMap::from([("gap".into(), Value::Integer(1))]),
                children: Vec::new(),
            },
        };

        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn validator_accepts_button_list_and_graph_props() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "VBox".into(),
                id: None,
                props: BTreeMap::new(),
                children: vec![
                    ComponentSpec {
                        kind: "Button".into(),
                        id: None,
                        props: BTreeMap::from([("text".into(), Value::String("Deploy".into()))]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "List".into(),
                        id: None,
                        props: BTreeMap::from([(
                            "items".into(),
                            Value::Array(vec![
                                Value::String("api".into()),
                                Value::String("jobs".into()),
                            ]),
                        )]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "Graph".into(),
                        id: None,
                        props: BTreeMap::from([(
                            "values".into(),
                            Value::Array(vec![Value::Integer(1), Value::Float("2.5".into())]),
                        )]),
                        children: Vec::new(),
                    },
                ],
            },
        };

        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn validator_accepts_table_columns_and_rows() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "Table".into(),
                id: Some("services".into()),
                props: BTreeMap::from([
                    (
                        "columns".into(),
                        Value::Array(vec![
                            Value::Object(BTreeMap::from([
                                ("key".into(), Value::String("service".into())),
                                ("title".into(), Value::String("Service".into())),
                                ("width".into(), Value::Integer(12)),
                            ])),
                            Value::Object(BTreeMap::from([
                                ("key".into(), Value::String("state".into())),
                                ("title".into(), Value::String("State".into())),
                                ("width".into(), Value::Integer(8)),
                                ("align".into(), Value::String("center".into())),
                            ])),
                        ]),
                    ),
                    (
                        "rows".into(),
                        Value::Array(vec![Value::Object(BTreeMap::from([
                            ("service".into(), Value::String("api".into())),
                            ("state".into(), Value::String("ok".into())),
                        ]))]),
                    ),
                ]),
                children: Vec::new(),
            },
        };

        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn validator_rejects_invalid_list_and_graph_arrays() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "VBox".into(),
                id: None,
                props: BTreeMap::new(),
                children: vec![
                    ComponentSpec {
                        kind: "List".into(),
                        id: None,
                        props: BTreeMap::from([(
                            "items".into(),
                            Value::Array(vec![Value::Integer(1)]),
                        )]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "Graph".into(),
                        id: None,
                        props: BTreeMap::from([(
                            "values".into(),
                            Value::Array(vec![Value::String("oops".into())]),
                        )]),
                        children: Vec::new(),
                    },
                ],
            },
        };

        let rendered = spec
            .validate()
            .expect_err("invalid arrays should fail")
            .to_string();
        assert!(rendered.contains(
            "root.children[0].props.items: property `items` must be an array of non-empty strings"
        ));
        assert!(rendered.contains(
            "root.children[1].props.values: property `values` must be an array of numbers"
        ));
    }

    #[test]
    fn validator_rejects_negative_gap_prop() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "HBox".into(),
                id: None,
                props: BTreeMap::from([("gap".into(), Value::Integer(-1))]),
                children: Vec::new(),
            },
        };

        let errors = spec.validate().expect_err("negative gap should fail");

        assert_eq!(errors.errors()[0].path, "root.props.gap");
    }

    #[test]
    fn validator_accepts_layout_constraint_props() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "HBox".into(),
                id: None,
                props: BTreeMap::from([
                    ("gap".into(), Value::Integer(1)),
                    ("align".into(), Value::String("center".into())),
                    ("justify".into(), Value::String("end".into())),
                ]),
                children: vec![
                    ComponentSpec {
                        kind: "Label".into(),
                        id: None,
                        props: BTreeMap::from([
                            ("text".into(), Value::String("Fixed".into())),
                            ("width".into(), Value::Integer(4)),
                        ]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "Label".into(),
                        id: None,
                        props: BTreeMap::from([
                            ("text".into(), Value::String("Grow".into())),
                            ("grow".into(), Value::Integer(2)),
                        ]),
                        children: Vec::new(),
                    },
                ],
            },
        };

        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn validator_rejects_invalid_layout_constraint_props() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "Label".into(),
                id: None,
                props: BTreeMap::from([
                    ("text".into(), Value::String("Hello".into())),
                    ("width_pct".into(), Value::Integer(120)),
                    ("grow".into(), Value::Integer(0)),
                ]),
                children: Vec::new(),
            },
        };

        let errors = spec
            .validate()
            .expect_err("invalid layout props should fail");
        let rendered = errors.to_string();

        assert!(rendered
            .contains("root.props.width_pct: property `width_pct` must be between 0 and 100"));
        assert!(rendered.contains("root.props.grow: property `grow` must be greater than zero"));
    }

    #[test]
    fn validator_rejects_invalid_stack_align_and_justify_props() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "VBox".into(),
                id: None,
                props: BTreeMap::from([
                    ("align".into(), Value::String("diagonal".into())),
                    ("justify".into(), Value::String("space-around".into())),
                ]),
                children: Vec::new(),
            },
        };

        let errors = spec
            .validate()
            .expect_err("invalid stack alignment props should fail");
        let rendered = errors.to_string();

        assert!(rendered.contains(
            "root.props.align: property `align` must be one of: start, center, end, stretch (got `diagonal`)"
        ));
        assert!(rendered.contains(
            "root.props.justify: property `justify` must be one of: start, center, end (got `space-around`)"
        ));
    }

    #[test]
    fn parses_dashboard_toml_example() {
        let input = std::fs::read_to_string(fixture_path("examples/dashboard.toml"))
            .expect("dashboard example should exist");

        let spec = AppSpec::from_toml_str(&input).expect("dashboard example should parse");

        assert_eq!(spec.theme.as_deref(), Some("dark"));
        assert_eq!(spec.root.kind, "Panel");
        assert_eq!(spec.root.children.len(), 3);
    }

    #[test]
    fn parses_dashboard_json_example() {
        let input = std::fs::read_to_string(fixture_path("examples/dashboard.json"))
            .expect("json example should exist");

        let spec = AppSpec::from_json_str(&input).expect("json example should parse");

        assert_eq!(spec.theme.as_deref(), Some("minimal"));
        assert_eq!(spec.root.kind, "Panel");
        assert_eq!(spec.root.children.len(), 4);
    }

    #[test]
    fn parses_showcase_layout_example() {
        let input = std::fs::read_to_string(fixture_path("examples/showcase-layout.toml"))
            .expect("showcase layout example should exist");

        let spec = AppSpec::from_toml_str(&input).expect("showcase layout example should parse");

        assert_eq!(spec.theme.as_deref(), Some("dark"));
        assert_eq!(spec.root.kind, "Panel");
        assert_eq!(spec.root.children.len(), 1);
        assert_eq!(spec.root.children[0].kind, "VBox");
        assert_eq!(spec.root.children[0].children.len(), 4);
    }

    #[test]
    fn parses_rich_dashboard_example() {
        let input = std::fs::read_to_string(fixture_path("examples/rich-dashboard.toml"))
            .expect("rich dashboard example should exist");

        let spec = AppSpec::from_toml_str(&input).expect("rich dashboard example should parse");

        assert_eq!(spec.theme.as_deref(), Some("dark"));
        assert_eq!(spec.root.kind, "Panel");
        assert_eq!(spec.root.id.as_deref(), Some("rich-dashboard"));
        assert_eq!(spec.root.children[0].kind, "VBox");
        assert!(spec.root.children[0]
            .children
            .iter()
            .any(|child| child.id.as_deref() == Some("detail-row")));
    }

    #[test]
    fn parses_redline_dashboard_example() {
        let input = std::fs::read_to_string(fixture_path("examples/redline-dashboard.toml"))
            .expect("redline dashboard example should exist");

        let spec = AppSpec::from_toml_str(&input).expect("redline dashboard example should parse");

        assert_eq!(spec.theme.as_deref(), Some("redline"));
        assert_eq!(spec.root.kind, "Panel");
        assert_eq!(spec.root.id.as_deref(), Some("redline-dashboard"));
        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn parses_table_demo_example() {
        let input = std::fs::read_to_string(fixture_path("examples/table-demo.toml"))
            .expect("table demo example should exist");

        let spec = AppSpec::from_toml_str(&input).expect("table demo example should parse");

        assert_eq!(spec.theme.as_deref(), Some("redline"));
        assert_eq!(spec.root.kind, "Panel");
        assert!(spec.root.children[0]
            .children
            .iter()
            .any(|child| child.kind == "Table"));
        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn parses_layout_pattern_examples() {
        for path in [
            "examples/layout-dense.toml",
            "examples/layout-sidebar.toml",
            "examples/layout-responsive.toml",
        ] {
            let input = std::fs::read_to_string(fixture_path(path))
                .expect("layout pattern example should exist");
            let spec = AppSpec::from_toml_str(&input).expect("layout pattern example should parse");

            assert_eq!(spec.schema_version, "0.1");
            assert!(matches!(spec.root.kind.as_str(), "Panel" | "VBox"));
            assert_eq!(spec.validate(), Ok(()));
        }
    }

    #[test]
    fn parses_interactive_flow_example() {
        let input = std::fs::read_to_string(fixture_path("examples/interactive-flow.toml"))
            .expect("interactive flow example should exist");
        let spec = AppSpec::from_toml_str(&input).expect("interactive flow example should parse");

        assert_eq!(spec.root.kind, "Panel");
        assert_eq!(spec.root.id.as_deref(), Some("interactive-flow"));
        spec.validate()
            .expect("interactive flow example should validate");
    }

    #[test]
    fn parses_cockpit_showcase_example() {
        let input = std::fs::read_to_string(fixture_path("examples/cockpit-showcase.toml"))
            .expect("cockpit showcase example should exist");
        let spec = AppSpec::from_toml_str(&input).expect("cockpit showcase example should parse");

        assert_eq!(spec.root.kind, "Panel");
        assert_eq!(spec.root.id.as_deref(), Some("cockpit"));
        spec.validate()
            .expect("cockpit showcase example should validate");
    }

    #[test]
    fn parses_visual_system_showcase_example() {
        let input = std::fs::read_to_string(fixture_path("examples/visual-system-showcase.toml"))
            .expect("visual system showcase example should exist");
        let spec =
            AppSpec::from_toml_str(&input).expect("visual system showcase example should parse");

        assert_eq!(spec.root.kind, "Panel");
        assert_eq!(spec.root.id.as_deref(), Some("visual-system"));
        spec.validate()
            .expect("visual system showcase example should validate");
    }

    #[test]
    fn parses_form_intent_example() {
        let input = std::fs::read_to_string(fixture_path("examples/form-intent.toml"))
            .expect("form intent example should exist");
        let spec = AppSpec::from_toml_str(&input).expect("form intent example should parse");

        assert_eq!(spec.forms.len(), 1);
        assert_eq!(spec.forms[0].id, "incident");
        spec.validate()
            .expect("form intent example should validate");
    }

    #[test]
    fn parses_embedded_device_control_example() {
        let input = std::fs::read_to_string(fixture_path("examples/embedded-device-control.toml"))
            .expect("embedded device control example should exist");
        let spec =
            AppSpec::from_toml_str(&input).expect("embedded device control example should parse");

        assert_eq!(spec.theme.as_deref(), Some("redline"));
        assert_eq!(spec.forms.len(), 1);
        assert_eq!(spec.forms[0].id, "device");
        assert_eq!(
            spec.data
                .as_ref()
                .expect("embedded device example should declare data sources")
                .sources
                .len(),
            1
        );
        assert_eq!(spec.actions.len(), 2);
        assert_eq!(spec.root.kind, "Panel");
        spec.validate()
            .expect("embedded device control example should validate");
    }

    #[test]
    fn parses_python_form_intent_json_contract() {
        let input = std::fs::read_to_string(fixture_path("examples/python/form-intent.json"))
            .expect("Python form intent JSON contract should exist");
        let spec =
            AppSpec::from_json_str(&input).expect("Python form intent JSON should parse as DSL");

        assert_eq!(spec.theme.as_deref(), Some("minimal"));
        assert_eq!(spec.forms.len(), 1);
        assert_eq!(spec.forms[0].id, "incident");
        assert_eq!(spec.actions.len(), 1);
        assert_eq!(spec.actions[0].id, "submit_incident");
        assert_eq!(spec.root.kind, "Panel");
        assert_eq!(spec.root.children[0].kind, "VBox");
        assert_eq!(spec.root.children[0].children[1].kind, "TextInput");
        spec.validate()
            .expect("Python form intent JSON should validate");
    }

    #[test]
    fn parses_application_templates() {
        for path in [
            "templates/operational-dashboard.toml",
            "templates/task-list.toml",
            "templates/metrics-monitor.toml",
        ] {
            let input =
                std::fs::read_to_string(fixture_path(path)).expect("template fixture should exist");
            let spec = AppSpec::from_toml_str(&input).expect("template should parse");

            assert_eq!(spec.schema_version, "0.1");
            assert!(spec
                .root
                .id
                .as_deref()
                .is_some_and(|id| id.starts_with("template-")));
            spec.validate().expect("template should validate");
        }
    }

    #[test]
    fn test_knob_and_fui_panel_validation() {
        let toml_input = r#"
            schema_version = "0.1"
            [root]
            kind = "Panel"
            id = "test-panel"
            [root.props]
            border_style = "hex"
            grid = true
            controls = true

            [[root.children]]
            kind = "Knob"
            id = "warp-dial"
            [root.children.props]
            value = 42.5
            min = 0.0
            max = 100.0
            title = "Warp Speed"
        "#;
        let spec = AppSpec::from_toml_str(toml_input).expect("spec should parse");
        assert_eq!(spec.validate(), Ok(()));

        // Let's test invalid properties
        let invalid_toml = r#"
            schema_version = "0.1"
            [root]
            kind = "Panel"
            [root.props]
            border_style = "invalid-style"
            grid = "yes"
        "#;
        let spec_invalid = AppSpec::from_toml_str(invalid_toml).expect("spec should parse");
        let errs = spec_invalid.validate().unwrap_err();
        assert!(errs.to_string().contains("border_style"));
        assert!(errs.to_string().contains("grid"));
    }

    #[test]
    fn validator_accepts_visual_panel_props() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: Some("redline".into()),
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "Panel".into(),
                id: Some("visual".into()),
                props: BTreeMap::from([
                    ("variant".into(), Value::String("hero".into())),
                    ("density".into(), Value::String("spacious".into())),
                    ("chrome".into(), Value::String("cinematic".into())),
                ]),
                children: Vec::new(),
            },
        };

        assert_eq!(spec.validate(), Ok(()));
    }

    #[test]
    fn validator_rejects_invalid_visual_panel_props() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: Some("redline".into()),
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "Panel".into(),
                id: Some("visual".into()),
                props: BTreeMap::from([
                    ("variant".into(), Value::String("loud".into())),
                    ("density".into(), Value::String("huge".into())),
                    ("chrome".into(), Value::String("glass".into())),
                ]),
                children: Vec::new(),
            },
        };

        let rendered = spec
            .validate()
            .expect_err("invalid visual panel props should fail")
            .to_string();

        assert!(rendered.contains("root.props.variant"));
        assert!(rendered.contains("root.props.density"));
        assert!(rendered.contains("root.props.chrome"));
    }
}
