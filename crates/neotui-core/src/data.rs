// Declarative data sources and bindings for runtime-driven TUI intent

use crate::dsl::{AppSpec, ComponentSpec, Value};
use crate::forms::{FormSpec, FormStore};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
#[cfg(feature = "http")]
use std::sync::mpsc::{self, Receiver, Sender};
#[cfg(feature = "http")]
use std::thread;
#[cfg(feature = "http")]
use std::time::{Duration, Instant};
#[cfg(feature = "http")]
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataSpec {
    pub sources: Vec<DataSourceSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionSpec {
    pub id: String,
    pub kind: ActionKind,
    pub http: HttpSourceSpec,
    pub refresh_sources: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActionKind {
    #[default]
    Http,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSourceSpec {
    Http(HttpSourceSpec),
}

impl DataSourceSpec {
    pub fn id(&self) -> &str {
        match self {
            Self::Http(source) => &source.id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpSourceSpec {
    pub id: String,
    pub url: String,
    pub method: HttpMethod,
    pub headers: BTreeMap<String, HttpHeaderValue>,
    pub body: Option<HttpBody>,
    pub timeout_ms: Option<u64>,
    pub refresh_ms: Option<u64>,
    pub retry_count: u16,
    pub retry_backoff_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
}

impl std::str::FromStr for HttpMethod {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "PATCH" => Ok(Self::Patch),
            "DELETE" => Ok(Self::Delete),
            _ => Err(format!(
                "method must be one of GET, POST, PUT, PATCH, DELETE (got `{value}`)"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpHeaderValue {
    Literal(String),
    Env {
        env: String,
        prefix: Option<String>,
        suffix: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpBody {
    Text(String),
    Json(Value),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataBinding {
    pub source: String,
    pub path: Vec<String>,
    pub status: bool,
}

impl DataBinding {
    pub fn parse(value: &str) -> Result<Self, String> {
        let mut parts = value.split('.').filter(|part| !part.is_empty());
        let Some(source) = parts.next() else {
            return Err("binding must start with a data source id".into());
        };
        let path = parts.map(ToOwned::to_owned).collect::<Vec<_>>();
        let status = path.first().is_some_and(|part| part == "$status");

        if source == "$status" {
            return Err("binding must include a data source id before `$status`".into());
        }

        Ok(Self {
            source: source.into(),
            path,
            status,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<DataValue>),
    Object(BTreeMap<String, DataValue>),
}

impl DataValue {
    pub fn get_path<'a>(&'a self, path: &[String]) -> Option<&'a DataValue> {
        let mut current = self;
        for part in path {
            match current {
                DataValue::Object(values) => current = values.get(part)?,
                DataValue::Array(values) => {
                    let index = part.parse::<usize>().ok()?;
                    current = values.get(index)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }

    pub fn display_string(&self) -> String {
        match self {
            Self::Null => String::new(),
            Self::Bool(value) => value.to_string(),
            Self::Number(value) => {
                let mut text = value.to_string();
                if text.ends_with(".0") {
                    text.truncate(text.len().saturating_sub(2));
                }
                text
            }
            Self::String(value) => value.clone(),
            Self::Array(_) | Self::Object(_) => String::new(),
        }
    }

    pub fn to_dsl_value(&self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Bool(value) => Value::Bool(*value),
            Self::Number(value) => Value::Float(value.to_string()),
            Self::String(value) => Value::String(value.clone()),
            Self::Array(values) => Value::Array(values.iter().map(Self::to_dsl_value).collect()),
            Self::Object(values) => Value::Object(
                values
                    .iter()
                    .map(|(key, value)| (key.clone(), value.to_dsl_value()))
                    .collect(),
            ),
        }
    }
}

impl From<serde_json::Value> for DataValue {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(value) => Self::Bool(value),
            serde_json::Value::Number(value) => Self::Number(value.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(value) => Self::String(value),
            serde_json::Value::Array(values) => {
                Self::Array(values.into_iter().map(Self::from).collect())
            }
            serde_json::Value::Object(values) => Self::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, Self::from(value)))
                    .collect(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataStatus {
    Idle,
    Loading,
    Stale,
    Ready,
    Error,
}

impl fmt::Display for DataStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Idle => "idle",
            Self::Loading => "loading",
            Self::Stale => "stale",
            Self::Ready => "ready",
            Self::Error => "error",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataSnapshot {
    pub status: DataStatus,
    pub value: Option<DataValue>,
    pub error: Option<String>,
}

impl DataSnapshot {
    pub fn idle() -> Self {
        Self {
            status: DataStatus::Idle,
            value: None,
            error: None,
        }
    }

    pub fn loading() -> Self {
        Self {
            status: DataStatus::Loading,
            value: None,
            error: None,
        }
    }

    pub fn ready(value: DataValue) -> Self {
        Self {
            status: DataStatus::Ready,
            value: Some(value),
            error: None,
        }
    }

    pub fn stale(value: DataValue) -> Self {
        Self {
            status: DataStatus::Stale,
            value: Some(value),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: DataStatus::Error,
            value: None,
            error: Some(message.into()),
        }
    }

    pub fn error_with_cached_value(message: impl Into<String>, value: DataValue) -> Self {
        Self {
            status: DataStatus::Error,
            value: Some(value),
            error: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataStore {
    values: HashMap<String, DataSnapshot>,
}

impl DataStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, id: impl Into<String>, snapshot: DataSnapshot) -> bool {
        let id = id.into();
        if self.values.get(&id) == Some(&snapshot) {
            return false;
        }
        self.values.insert(id, snapshot);
        true
    }

    pub fn get(&self, id: &str) -> Option<&DataSnapshot> {
        self.values.get(id)
    }

    pub fn resolve_binding(&self, binding: &DataBinding) -> Option<Value> {
        let snapshot = self.get(&binding.source)?;
        if binding.status {
            return Some(Value::String(snapshot.status.to_string()));
        }
        let value = snapshot.value.as_ref()?.get_path(&binding.path)?;
        Some(value.to_dsl_value())
    }
}

impl fmt::Display for ActionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Idle => "idle",
            Self::Loading => "loading",
            Self::Ready => "ready",
            Self::Error => "error",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionSnapshot {
    pub status: ActionStatus,
    pub error: Option<String>,
}

impl ActionSnapshot {
    pub fn idle() -> Self {
        Self {
            status: ActionStatus::Idle,
            error: None,
        }
    }

    pub fn loading() -> Self {
        Self {
            status: ActionStatus::Loading,
            error: None,
        }
    }

    pub fn ready() -> Self {
        Self {
            status: ActionStatus::Ready,
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: ActionStatus::Error,
            error: Some(message.into()),
        }
    }

    pub fn from_runtime_update(update: &ActionRuntimeUpdate) -> Self {
        match update.status {
            ActionStatus::Idle => Self::idle(),
            ActionStatus::Loading => Self::loading(),
            ActionStatus::Ready => Self::ready(),
            ActionStatus::Error => Self::error(
                update
                    .error
                    .clone()
                    .unwrap_or_else(|| "action failed".into()),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionStore {
    values: HashMap<String, ActionSnapshot>,
}

impl ActionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, id: impl Into<String>, snapshot: ActionSnapshot) -> bool {
        let id = id.into();
        if self.values.get(&id) == Some(&snapshot) {
            return false;
        }
        self.values.insert(id, snapshot);
        true
    }

    pub fn get(&self, id: &str) -> Option<&ActionSnapshot> {
        self.values.get(id)
    }

    pub fn resolve_binding(&self, binding: &DataBinding) -> Option<Value> {
        if binding.source != "$actions" {
            return None;
        }
        let action_id = binding.path.first()?;
        let snapshot = self.get(action_id)?;
        match binding.path.get(1).map(String::as_str) {
            Some("$status") => Some(Value::String(snapshot.status.to_string())),
            Some("error") => Some(Value::String(snapshot.error.clone().unwrap_or_default())),
            _ => None,
        }
    }
}

pub fn apply_data_bindings(spec: &AppSpec, store: &DataStore) -> AppSpec {
    apply_runtime_bindings(spec, store, &ActionStore::new())
}

pub fn apply_runtime_bindings(
    spec: &AppSpec,
    data_store: &DataStore,
    action_store: &ActionStore,
) -> AppSpec {
    apply_runtime_bindings_with_forms(spec, data_store, action_store, &FormStore::new())
}

pub fn apply_runtime_bindings_with_forms(
    spec: &AppSpec,
    data_store: &DataStore,
    action_store: &ActionStore,
    form_store: &FormStore,
) -> AppSpec {
    let mut next = spec.clone();
    let store = BindingStore {
        data: data_store,
        actions: action_store,
        forms: form_store,
    };
    next.root = apply_component_bindings(&spec.root, &store);
    next
}

struct BindingStore<'a> {
    data: &'a DataStore,
    actions: &'a ActionStore,
    forms: &'a FormStore,
}

impl BindingStore<'_> {
    fn resolve_binding(&self, binding: &DataBinding) -> Option<Value> {
        if binding.source == "$actions" {
            self.actions.resolve_binding(binding)
        } else if binding.source == "$forms" {
            self.forms.resolve_binding(binding)
        } else {
            self.data.resolve_binding(binding)
        }
    }

    fn fallback_value(&self, target_prop: &str, binding: &DataBinding) -> Value {
        if binding.source == "$actions" {
            let action_id = binding.path.first().map(String::as_str);
            return action_binding_fallback_value(
                target_prop,
                action_id.and_then(|id| self.actions.get(id)),
            );
        }
        if binding.source == "$forms" {
            return form_binding_fallback_value(target_prop);
        }
        binding_fallback_value(target_prop, self.data.get(&binding.source))
    }
}

fn apply_component_bindings(component: &ComponentSpec, store: &BindingStore<'_>) -> ComponentSpec {
    let mut next = component.clone();
    let bindings = [
        ("text_from", "text"),
        ("value_from", "value"),
        ("items_from", "items"),
        ("values_from", "values"),
        ("rows_from", "rows"),
        ("status_from", "status"),
    ];

    for (binding_prop, target_prop) in bindings {
        let Some(Value::String(binding_text)) = next.props.get(binding_prop) else {
            continue;
        };
        let Ok(binding) = DataBinding::parse(binding_text) else {
            continue;
        };
        let value = store
            .resolve_binding(&binding)
            .unwrap_or_else(|| store.fallback_value(target_prop, &binding));
        next.props.insert(
            target_prop.into(),
            coerce_binding_value(&next.kind, target_prop, value),
        );
    }

    next.children = component
        .children
        .iter()
        .map(|child| apply_component_bindings(child, store))
        .collect();
    next
}

fn action_binding_fallback_value(target_prop: &str, snapshot: Option<&ActionSnapshot>) -> Value {
    let text = snapshot
        .map(|snapshot| snapshot.status.to_string())
        .unwrap_or_else(|| "idle".into());

    match target_prop {
        "items" | "values" | "rows" => Value::Array(Vec::new()),
        "status" => Value::String(text),
        _ => Value::String(text),
    }
}

fn form_binding_fallback_value(target_prop: &str) -> Value {
    match target_prop {
        "items" | "values" | "rows" => Value::Array(Vec::new()),
        "status" => Value::String("info".into()),
        _ => Value::String(String::new()),
    }
}

fn binding_fallback_value(target_prop: &str, snapshot: Option<&DataSnapshot>) -> Value {
    let text = match snapshot.map(|snapshot| &snapshot.status) {
        Some(DataStatus::Loading) => "loading",
        Some(DataStatus::Stale) => "stale",
        Some(DataStatus::Error) => "error",
        Some(DataStatus::Idle) | None => "idle",
        Some(DataStatus::Ready) => "",
    };

    match target_prop {
        "items" | "values" | "rows" => Value::Array(Vec::new()),
        "status" => Value::String(text.into()),
        _ => Value::String(text.into()),
    }
}

fn coerce_binding_value(component_kind: &str, target_prop: &str, value: Value) -> Value {
    match target_prop {
        "value" if component_kind == "Gauge" => match value {
            Value::Integer(_) | Value::Float(_) => value,
            _ => Value::Integer(0),
        },
        "items" => match value {
            Value::Array(values) => Value::Array(values.into_iter().map(value_to_string).collect()),
            other => Value::Array(vec![value_to_string(other)]),
        },
        "values" => match value {
            Value::Array(values) => Value::Array(values),
            other => Value::Array(vec![other]),
        },
        "status" => coerce_status_value(value),
        "text" | "value" => value_to_string(value),
        _ => value,
    }
}

fn coerce_status_value(value: Value) -> Value {
    let Value::String(value) = value_to_string(value) else {
        return Value::String("info".into());
    };
    let status = match value.as_str() {
        "ready" => "success",
        "loading" | "idle" => "info",
        "stale" => "warning",
        "error" => "danger",
        other => other,
    };
    Value::String(status.into())
}

fn value_to_string(value: Value) -> Value {
    match value {
        Value::String(_) => value,
        Value::Null => Value::String(String::new()),
        Value::Bool(value) => Value::String(value.to_string()),
        Value::Integer(value) => Value::String(value.to_string()),
        Value::Float(value) => Value::String(value),
        Value::Array(_) | Value::Object(_) => Value::String(String::new()),
    }
}

#[derive(Debug)]
pub enum DataRuntimeError {
    MissingEnv { source_id: String, env: String },
    Http { source_id: String, message: String },
    Unsupported,
}

impl fmt::Display for DataRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnv { source_id, env } => {
                write!(
                    f,
                    "data source `{source_id}` requires environment variable `{env}`"
                )
            }
            Self::Http { source_id, message } => {
                write!(f, "data source `{source_id}` request failed: {message}")
            }
            Self::Unsupported => write!(f, "HTTP data runtime is not enabled"),
        }
    }
}

impl std::error::Error for DataRuntimeError {}

#[derive(Debug, Clone)]
pub struct DataRuntimeUpdate {
    pub source_id: String,
    pub snapshot: DataSnapshot,
    #[allow(dead_code)]
    generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionStatus {
    Idle,
    Loading,
    Ready,
    Error,
}

#[derive(Debug, Clone)]
pub struct ActionRuntimeUpdate {
    pub action_id: String,
    pub status: ActionStatus,
    pub error: Option<String>,
    pub refresh_sources: Vec<String>,
    #[allow(dead_code)]
    generation: u64,
}

#[cfg(feature = "http")]
#[derive(Debug)]
pub struct HttpDataRuntime {
    sources: Vec<HttpSourceSpec>,
    tx: Sender<DataRuntimeUpdate>,
    rx: Receiver<DataRuntimeUpdate>,
    next_refresh: HashMap<String, Instant>,
    in_flight: HashMap<String, bool>,
    generation: HashMap<String, u64>,
    snapshots: HashMap<String, DataSnapshot>,
}

#[cfg(feature = "http")]
impl HttpDataRuntime {
    pub fn new(spec: &DataSpec) -> Self {
        let (tx, rx) = mpsc::channel();
        let sources = spec
            .sources
            .iter()
            .map(|source| match source {
                DataSourceSpec::Http(source) => source.clone(),
            })
            .collect();

        Self {
            sources,
            tx,
            rx,
            next_refresh: HashMap::new(),
            in_flight: HashMap::new(),
            generation: HashMap::new(),
            snapshots: HashMap::new(),
        }
    }

    pub fn poll(&mut self) -> Vec<DataRuntimeUpdate> {
        let mut updates = Vec::new();
        while let Ok(update) = self.rx.try_recv() {
            let current_generation = self.generation.get(&update.source_id).copied().unwrap_or(0);
            if update.generation != current_generation {
                debug!(
                    target: "neotui::data",
                    source_id = update.source_id.as_str(),
                    "discarding stale HTTP data source response"
                );
                continue;
            }

            self.in_flight.insert(update.source_id.clone(), false);
            let mut snapshot = update.snapshot;
            if snapshot.status == DataStatus::Error {
                if let Some(cached_value) = self
                    .snapshots
                    .get(&update.source_id)
                    .and_then(|snapshot| snapshot.value.clone())
                {
                    snapshot = DataSnapshot::error_with_cached_value(
                        snapshot.error.unwrap_or_else(|| "request failed".into()),
                        cached_value,
                    );
                }
            }
            self.snapshots
                .insert(update.source_id.clone(), snapshot.clone());
            updates.push(DataRuntimeUpdate {
                source_id: update.source_id,
                snapshot,
                generation: update.generation,
            });
        }
        updates
    }

    pub fn tick(&mut self) -> Vec<DataRuntimeUpdate> {
        let mut updates = self.poll();
        let now = Instant::now();
        for source in self.sources.clone() {
            let due = match self.next_refresh.get(&source.id) {
                Some(next) => *next <= now,
                None => true,
            };
            let busy = self.in_flight.get(&source.id).copied().unwrap_or(false);
            if due && !busy {
                let next_generation = self.generation.get(&source.id).copied().unwrap_or(0) + 1;
                self.generation.insert(source.id.clone(), next_generation);
                let lifecycle_snapshot = self
                    .snapshots
                    .get(&source.id)
                    .and_then(|snapshot| snapshot.value.clone())
                    .map(DataSnapshot::stale)
                    .unwrap_or_else(DataSnapshot::loading);
                self.snapshots
                    .insert(source.id.clone(), lifecycle_snapshot.clone());
                updates.push(DataRuntimeUpdate {
                    source_id: source.id.clone(),
                    snapshot: lifecycle_snapshot,
                    generation: next_generation,
                });

                self.spawn_request(source.clone(), next_generation);
                self.in_flight.insert(source.id.clone(), true);
                let delay = Duration::from_millis(source.refresh_ms.unwrap_or(0).max(1_000));
                self.next_refresh.insert(source.id.clone(), now + delay);
            }
        }
        updates
    }

    pub fn refresh_sources(&mut self, source_ids: &[String]) -> Vec<DataRuntimeUpdate> {
        let mut updates = self.poll();
        let now = Instant::now();
        for source_id in source_ids {
            let Some(source) = self
                .sources
                .iter()
                .find(|source| &source.id == source_id)
                .cloned()
            else {
                continue;
            };
            let busy = self.in_flight.get(&source.id).copied().unwrap_or(false);
            if busy {
                continue;
            }
            let next_generation = self.generation.get(&source.id).copied().unwrap_or(0) + 1;
            self.generation.insert(source.id.clone(), next_generation);
            let lifecycle_snapshot = self
                .snapshots
                .get(&source.id)
                .and_then(|snapshot| snapshot.value.clone())
                .map(DataSnapshot::stale)
                .unwrap_or_else(DataSnapshot::loading);
            self.snapshots
                .insert(source.id.clone(), lifecycle_snapshot.clone());
            updates.push(DataRuntimeUpdate {
                source_id: source.id.clone(),
                snapshot: lifecycle_snapshot,
                generation: next_generation,
            });

            self.spawn_request(source.clone(), next_generation);
            self.in_flight.insert(source.id.clone(), true);
            let delay = Duration::from_millis(source.refresh_ms.unwrap_or(0).max(1_000));
            self.next_refresh.insert(source.id.clone(), now + delay);
        }
        updates
    }

    fn spawn_request(&self, source: HttpSourceSpec, generation: u64) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let source_id = source.id.clone();
            let snapshot = match execute_http_source_with_retry(&source) {
                Ok(value) => DataSnapshot::ready(value),
                Err(err) => DataSnapshot::error(err.to_string()),
            };
            let _ = tx.send(DataRuntimeUpdate {
                source_id,
                snapshot,
                generation,
            });
        });
    }
}

#[cfg(feature = "http")]
#[derive(Debug)]
pub struct HttpActionRuntime {
    actions: HashMap<String, ActionSpec>,
    tx: Sender<ActionRuntimeUpdate>,
    rx: Receiver<ActionRuntimeUpdate>,
    in_flight: HashMap<String, bool>,
    generation: HashMap<String, u64>,
}

#[cfg(feature = "http")]
impl HttpActionRuntime {
    pub fn new(actions: &[ActionSpec]) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            actions: actions
                .iter()
                .map(|action| (action.id.clone(), action.clone()))
                .collect(),
            tx,
            rx,
            in_flight: HashMap::new(),
            generation: HashMap::new(),
        }
    }

    pub fn trigger(&mut self, action_id: &str) -> Option<ActionRuntimeUpdate> {
        self.trigger_with_forms(action_id, &FormStore::new())
    }

    pub fn trigger_with_forms(
        &mut self,
        action_id: &str,
        forms: &FormStore,
    ) -> Option<ActionRuntimeUpdate> {
        self.trigger_with_form_specs(action_id, forms, &[])
    }

    pub fn trigger_with_form_specs(
        &mut self,
        action_id: &str,
        forms: &FormStore,
        form_specs: &[FormSpec],
    ) -> Option<ActionRuntimeUpdate> {
        let action = self.actions.get(action_id)?.clone();
        if self.in_flight.get(action_id).copied().unwrap_or(false) {
            return None;
        }

        let generation = self.generation.get(action_id).copied().unwrap_or(0) + 1;
        self.generation.insert(action_id.into(), generation);
        if let Some(message) = validate_required_form_payload(&action, forms, form_specs) {
            return Some(ActionRuntimeUpdate {
                action_id: action.id,
                status: ActionStatus::Error,
                error: Some(message),
                refresh_sources: Vec::new(),
                generation,
            });
        }

        let action = render_action_payload(&action, forms);
        self.in_flight.insert(action_id.into(), true);
        self.spawn_action(action.clone(), generation);
        Some(ActionRuntimeUpdate {
            action_id: action.id,
            status: ActionStatus::Loading,
            error: None,
            refresh_sources: Vec::new(),
            generation,
        })
    }

    pub fn poll(&mut self) -> Vec<ActionRuntimeUpdate> {
        let mut updates = Vec::new();
        while let Ok(update) = self.rx.try_recv() {
            let current_generation = self.generation.get(&update.action_id).copied().unwrap_or(0);
            if update.generation != current_generation {
                continue;
            }
            self.in_flight.insert(update.action_id.clone(), false);
            updates.push(update);
        }
        updates
    }

    fn spawn_action(&self, action: ActionSpec, generation: u64) {
        let tx = self.tx.clone();
        thread::spawn(move || {
            let (status, error, refresh_sources) =
                match execute_http_source_with_retry(&action.http) {
                    Ok(_) => (ActionStatus::Ready, None, action.refresh_sources.clone()),
                    Err(error) => (ActionStatus::Error, Some(error.to_string()), Vec::new()),
                };
            let _ = tx.send(ActionRuntimeUpdate {
                action_id: action.id,
                status,
                error,
                refresh_sources,
                generation,
            });
        });
    }
}

#[cfg(any(test, feature = "http"))]
fn validate_required_form_payload(
    action: &ActionSpec,
    forms: &FormStore,
    form_specs: &[FormSpec],
) -> Option<String> {
    let body = action.http.body.as_ref()?;
    let mut bindings = Vec::new();
    collect_form_templates_from_body(body, &mut bindings);

    for binding in bindings {
        let Some((form_id, field_id)) = form_binding_parts(&binding) else {
            continue;
        };
        let required = form_specs.iter().any(|form| {
            form.id == form_id
                && form
                    .fields
                    .iter()
                    .any(|field| field.id == field_id && field.required)
        });
        if required {
            let value = forms.get(&form_id, &field_id);
            if value.map(is_empty_form_value).unwrap_or(true) {
                return Some(format!(
                    "required form field `{form_id}.{field_id}` is empty"
                ));
            }
        }
    }

    None
}

#[cfg(any(test, feature = "http"))]
fn collect_form_templates_from_body(body: &HttpBody, output: &mut Vec<String>) {
    match body {
        HttpBody::Text(text) => collect_form_template_text(text, output),
        HttpBody::Json(value) => collect_form_templates_from_value(value, output),
    }
}

#[cfg(any(test, feature = "http"))]
fn collect_form_templates_from_value(value: &Value, output: &mut Vec<String>) {
    match value {
        Value::String(text) => collect_form_template_text(text, output),
        Value::Array(values) => {
            for value in values {
                collect_form_templates_from_value(value, output);
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                collect_form_templates_from_value(value, output);
            }
        }
        _ => {}
    }
}

#[cfg(any(test, feature = "http"))]
fn collect_form_template_text(text: &str, output: &mut Vec<String>) {
    if text.starts_with("$forms.") {
        output.push(text.to_string());
    }
}

#[cfg(any(test, feature = "http"))]
fn form_binding_parts(binding_text: &str) -> Option<(String, String)> {
    let binding = DataBinding::parse(binding_text).ok()?;
    if binding.source != "$forms" || binding.path.len() != 2 {
        return None;
    }
    Some((binding.path[0].clone(), binding.path[1].clone()))
}

#[cfg(any(test, feature = "http"))]
fn is_empty_form_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(value) => value.trim().is_empty(),
        Value::Array(values) => values.is_empty(),
        Value::Object(values) => values.is_empty(),
        _ => false,
    }
}

pub fn render_action_payload(action: &ActionSpec, forms: &FormStore) -> ActionSpec {
    let mut action = action.clone();
    action.http.body = action
        .http
        .body
        .as_ref()
        .map(|body| render_http_body_templates(body, forms));
    action
}

fn render_http_body_templates(body: &HttpBody, forms: &FormStore) -> HttpBody {
    match body {
        HttpBody::Text(text) => HttpBody::Text(render_text_template(text, forms)),
        HttpBody::Json(value) => HttpBody::Json(render_value_templates(value, forms)),
    }
}

fn render_value_templates(value: &Value, forms: &FormStore) -> Value {
    match value {
        Value::String(text) => resolve_form_template(text, forms)
            .cloned()
            .unwrap_or_else(|| Value::String(text.clone())),
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| render_value_templates(value, forms))
                .collect(),
        ),
        Value::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), render_value_templates(value, forms)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn render_text_template(text: &str, forms: &FormStore) -> String {
    resolve_form_template(text, forms)
        .map(|value| value_to_string(value.clone()))
        .and_then(|value| match value {
            Value::String(text) => Some(text),
            _ => None,
        })
        .unwrap_or_else(|| text.to_string())
}

fn resolve_form_template<'a>(text: &str, forms: &'a FormStore) -> Option<&'a Value> {
    if !text.starts_with("$forms.") {
        return None;
    }
    let binding = DataBinding::parse(text).ok()?;
    if binding.path.len() != 2 {
        return None;
    }
    forms.get(binding.path.first()?, binding.path.get(1)?)
}

#[cfg(feature = "http")]
fn execute_http_source_with_retry(source: &HttpSourceSpec) -> Result<DataValue, DataRuntimeError> {
    let attempts = source.retry_count.saturating_add(1);
    let mut last_error = None;

    for attempt in 0..attempts {
        match execute_http_source(source) {
            Ok(value) => return Ok(value),
            Err(error) => {
                last_error = Some(error);
                if attempt + 1 < attempts && source.retry_backoff_ms > 0 {
                    thread::sleep(Duration::from_millis(source.retry_backoff_ms));
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| DataRuntimeError::Http {
        source_id: source.id.clone(),
        message: "request failed".into(),
    }))
}

#[cfg(feature = "http")]
fn execute_http_source(source: &HttpSourceSpec) -> Result<DataValue, DataRuntimeError> {
    let timeout = Duration::from_millis(source.timeout_ms.unwrap_or(5_000));
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build();
    let agent = ureq::Agent::new_with_config(config);
    debug!(
        target: "neotui::data",
        source_id = source.id.as_str(),
        method = source.method.as_str(),
        "dispatching HTTP data source"
    );

    let response = match source.method {
        HttpMethod::Get => {
            let request = apply_headers(agent.get(source.url.as_str()), source)?;
            request.call()
        }
        HttpMethod::Delete => {
            let request = apply_headers(agent.delete(source.url.as_str()), source)?;
            if source.body.is_some() {
                send_request_body(request.force_send_body(), source.body.as_ref())
            } else {
                request.call()
            }
        }
        HttpMethod::Post => {
            let request = apply_headers(agent.post(source.url.as_str()), source)?;
            send_request_body(request, source.body.as_ref())
        }
        HttpMethod::Put => {
            let request = apply_headers(agent.put(source.url.as_str()), source)?;
            send_request_body(request, source.body.as_ref())
        }
        HttpMethod::Patch => {
            let request = apply_headers(agent.patch(source.url.as_str()), source)?;
            send_request_body(request, source.body.as_ref())
        }
    }
    .map_err(|source_error| DataRuntimeError::Http {
        source_id: source.id.clone(),
        message: source_error.to_string(),
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(DataRuntimeError::Http {
            source_id: source.id.clone(),
            message: format!("status {}", status.as_u16()),
        });
    }

    let json = response
        .into_body()
        .read_json::<serde_json::Value>()
        .map_err(|source_error| DataRuntimeError::Http {
            source_id: source.id.clone(),
            message: source_error.to_string(),
        })?;

    Ok(DataValue::from(json))
}

#[cfg(feature = "http")]
fn apply_headers<B>(
    mut request: ureq::RequestBuilder<B>,
    source: &HttpSourceSpec,
) -> Result<ureq::RequestBuilder<B>, DataRuntimeError> {
    for (name, value) in &source.headers {
        let header_value = resolve_header_value(&source.id, value)?;
        request = request.header(name.as_str(), header_value);
    }
    Ok(request)
}

#[cfg(feature = "http")]
fn send_request_body(
    request: ureq::RequestBuilder<ureq::typestate::WithBody>,
    body: Option<&HttpBody>,
) -> Result<ureq::http::Response<ureq::Body>, ureq::Error> {
    match body {
        Some(HttpBody::Text(body)) => request.send(body.as_str()),
        Some(HttpBody::Json(body)) => request.send_json(value_to_json(body)),
        None => request.send_empty(),
    }
}

#[cfg(feature = "http")]
fn resolve_header_value(
    source_id: &str,
    value: &HttpHeaderValue,
) -> Result<String, DataRuntimeError> {
    match value {
        HttpHeaderValue::Literal(value) => Ok(value.clone()),
        HttpHeaderValue::Env {
            env,
            prefix,
            suffix,
        } => {
            let raw = std::env::var(env).map_err(|_| DataRuntimeError::MissingEnv {
                source_id: source_id.into(),
                env: env.clone(),
            })?;
            Ok(format!(
                "{}{}{}",
                prefix.as_deref().unwrap_or(""),
                raw,
                suffix.as_deref().unwrap_or("")
            ))
        }
    }
}

pub fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(value) => serde_json::Value::Bool(*value),
        Value::Integer(value) => serde_json::Value::Number((*value).into()),
        Value::Float(value) => value
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(value) => serde_json::Value::String(value.clone()),
        Value::Array(values) => {
            serde_json::Value::Array(values.iter().map(value_to_json).collect())
        }
        Value::Object(values) => serde_json::Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), value_to_json(value)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::{AppSpec, ComponentSpec};

    #[test]
    fn data_binding_parses_source_path_and_status() {
        let binding = DataBinding::parse("ops.metrics.cpu").expect("binding should parse");
        assert_eq!(binding.source, "ops");
        assert_eq!(binding.path, vec!["metrics", "cpu"]);
        assert!(!binding.status);

        let status = DataBinding::parse("ops.$status").expect("status binding should parse");
        assert_eq!(status.source, "ops");
        assert!(status.status);
    }

    #[test]
    fn data_store_resolves_nested_json_paths() {
        let mut object = BTreeMap::new();
        object.insert("summary".into(), DataValue::String("backend ready".into()));
        object.insert("cpu".into(), DataValue::Number(42.0));
        let mut store = DataStore::new();
        let _ = store.set("ops", DataSnapshot::ready(DataValue::Object(object)));

        let summary = store
            .resolve_binding(&DataBinding::parse("ops.summary").unwrap())
            .expect("summary should resolve");
        let cpu = store
            .resolve_binding(&DataBinding::parse("ops.cpu").unwrap())
            .expect("cpu should resolve");

        assert_eq!(summary, Value::String("backend ready".into()));
        assert_eq!(cpu, Value::Float("42".into()));
    }

    #[test]
    fn apply_data_bindings_fills_widget_props_from_store() {
        let mut root_props = BTreeMap::new();
        root_props.insert("text_from".into(), Value::String("ops.summary".into()));
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "Label".into(),
                id: None,
                props: root_props,
                children: Vec::new(),
            },
        };
        let mut object = BTreeMap::new();
        object.insert("summary".into(), DataValue::String("live".into()));
        let mut store = DataStore::new();
        let _ = store.set("ops", DataSnapshot::ready(DataValue::Object(object)));

        let effective = apply_data_bindings(&spec, &store);

        assert_eq!(
            effective.root.props.get("text"),
            Some(&Value::String("live".into()))
        );
    }

    #[test]
    fn apply_data_bindings_uses_loading_fallback() {
        let mut root_props = BTreeMap::new();
        root_props.insert("text_from".into(), Value::String("ops.summary".into()));
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "Label".into(),
                id: None,
                props: root_props,
                children: Vec::new(),
            },
        };
        let mut store = DataStore::new();
        let _ = store.set("ops", DataSnapshot::loading());

        let effective = apply_data_bindings(&spec, &store);

        assert_eq!(
            effective.root.props.get("text"),
            Some(&Value::String("loading".into()))
        );
    }

    #[test]
    fn apply_runtime_bindings_resolves_action_status() {
        let mut root_props = BTreeMap::new();
        root_props.insert("text".into(), Value::String("action idle".into()));
        root_props.insert(
            "text_from".into(),
            Value::String("$actions.refresh_now.$status".into()),
        );
        root_props.insert(
            "status_from".into(),
            Value::String("$actions.refresh_now.$status".into()),
        );
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "StatusStrip".into(),
                id: None,
                props: root_props,
                children: Vec::new(),
            },
        };
        let mut actions = ActionStore::new();
        let _ = actions.set("refresh_now", ActionSnapshot::loading());

        let effective = apply_runtime_bindings(&spec, &DataStore::new(), &actions);

        assert_eq!(
            effective.root.props.get("text"),
            Some(&Value::String("loading".into()))
        );
        assert_eq!(
            effective.root.props.get("status"),
            Some(&Value::String("info".into()))
        );
    }

    #[test]
    fn apply_runtime_bindings_resolves_form_values() {
        let mut root_props = BTreeMap::new();
        root_props.insert(
            "text_from".into(),
            Value::String("$forms.incident.summary".into()),
        );
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "Label".into(),
                id: None,
                props: root_props,
                children: Vec::new(),
            },
        };
        let mut forms = FormStore::new();
        let _ = forms.set("incident", "summary", Value::String("Disk full".into()));

        let effective = apply_runtime_bindings_with_forms(
            &spec,
            &DataStore::new(),
            &ActionStore::new(),
            &forms,
        );

        assert_eq!(
            effective.root.props.get("text"),
            Some(&Value::String("Disk full".into()))
        );
    }

    #[test]
    fn render_action_payload_resolves_form_json_templates() {
        let action = ActionSpec {
            id: "submit_incident".into(),
            kind: ActionKind::Http,
            http: HttpSourceSpec {
                id: "submit_incident".into(),
                url: "http://127.0.0.1:7878/ack".into(),
                method: HttpMethod::Post,
                headers: BTreeMap::new(),
                body: Some(HttpBody::Json(Value::Object(BTreeMap::from([
                    (
                        "summary".into(),
                        Value::String("$forms.incident.summary".into()),
                    ),
                    ("static".into(), Value::String("kept".into())),
                ])))),
                timeout_ms: None,
                refresh_ms: None,
                retry_count: 0,
                retry_backoff_ms: 0,
            },
            refresh_sources: Vec::new(),
        };
        let mut forms = FormStore::new();
        let _ = forms.set("incident", "summary", Value::String("Disk full".into()));

        let rendered = render_action_payload(&action, &forms);

        assert_eq!(
            rendered.http.body,
            Some(HttpBody::Json(Value::Object(BTreeMap::from([
                ("summary".into(), Value::String("Disk full".into())),
                ("static".into(), Value::String("kept".into())),
            ]))))
        );
    }

    #[test]
    fn required_form_payload_reports_empty_value_before_http_dispatch() {
        let action = ActionSpec {
            id: "submit_incident".into(),
            kind: ActionKind::Http,
            http: HttpSourceSpec {
                id: "submit_incident".into(),
                url: "http://127.0.0.1:7878/ack".into(),
                method: HttpMethod::Post,
                headers: BTreeMap::new(),
                body: Some(HttpBody::Json(Value::Object(BTreeMap::from([(
                    "summary".into(),
                    Value::String("$forms.incident.summary".into()),
                )])))),
                timeout_ms: None,
                refresh_ms: None,
                retry_count: 0,
                retry_backoff_ms: 0,
            },
            refresh_sources: Vec::new(),
        };
        let form_specs = vec![crate::forms::FormSpec {
            id: "incident".into(),
            fields: vec![crate::forms::FormFieldSpec {
                id: "summary".into(),
                kind: crate::forms::FormFieldKind::Text,
                initial: None,
                required: true,
            }],
        }];
        let mut forms = FormStore::new();
        let _ = forms.set("incident", "summary", Value::String(" ".into()));

        let message = validate_required_form_payload(&action, &forms, &form_specs)
            .expect("required empty field should fail");

        assert_eq!(message, "required form field `incident.summary` is empty");
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_action_runtime_rejects_invalid_required_form_payload_without_worker() {
        let action = ActionSpec {
            id: "submit_incident".into(),
            kind: ActionKind::Http,
            http: HttpSourceSpec {
                id: "submit_incident".into(),
                url: "http://127.0.0.1:1/ack".into(),
                method: HttpMethod::Post,
                headers: BTreeMap::new(),
                body: Some(HttpBody::Json(Value::Object(BTreeMap::from([(
                    "summary".into(),
                    Value::String("$forms.incident.summary".into()),
                )])))),
                timeout_ms: Some(1),
                refresh_ms: None,
                retry_count: 0,
                retry_backoff_ms: 0,
            },
            refresh_sources: Vec::new(),
        };
        let form_specs = vec![crate::forms::FormSpec {
            id: "incident".into(),
            fields: vec![crate::forms::FormFieldSpec {
                id: "summary".into(),
                kind: crate::forms::FormFieldKind::Text,
                initial: None,
                required: true,
            }],
        }];
        let mut forms = FormStore::new();
        let _ = forms.set("incident", "summary", Value::String(String::new()));
        let mut runtime = HttpActionRuntime::new(&[action]);

        let update = runtime
            .trigger_with_form_specs("submit_incident", &forms, &form_specs)
            .expect("known action should produce validation update");

        assert_eq!(update.status, ActionStatus::Error);
        assert_eq!(
            update.error.as_deref(),
            Some("required form field `incident.summary` is empty")
        );
        assert!(runtime.poll().is_empty());
    }

    #[test]
    fn render_action_payload_resolves_form_text_templates() {
        let action = ActionSpec {
            id: "submit_incident".into(),
            kind: ActionKind::Http,
            http: HttpSourceSpec {
                id: "submit_incident".into(),
                url: "http://127.0.0.1:7878/ack".into(),
                method: HttpMethod::Post,
                headers: BTreeMap::new(),
                body: Some(HttpBody::Text("$forms.incident.summary".into())),
                timeout_ms: None,
                refresh_ms: None,
                retry_count: 0,
                retry_backoff_ms: 0,
            },
            refresh_sources: Vec::new(),
        };
        let mut forms = FormStore::new();
        let _ = forms.set("incident", "summary", Value::String("Disk full".into()));

        let rendered = render_action_payload(&action, &forms);

        assert_eq!(rendered.http.body, Some(HttpBody::Text("Disk full".into())));
    }

    #[test]
    fn status_bindings_map_lifecycle_to_visual_statuses() {
        let mut root_props = BTreeMap::new();
        root_props.insert("text".into(), Value::String("status".into()));
        root_props.insert("status_from".into(), Value::String("ops.$status".into()));
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            data: None,
            actions: Vec::new(),
            forms: Vec::new(),
            root: ComponentSpec {
                kind: "StatusStrip".into(),
                id: None,
                props: root_props,
                children: Vec::new(),
            },
        };
        let mut store = DataStore::new();
        let _ = store.set(
            "ops",
            DataSnapshot::stale(DataValue::String("cached".into())),
        );

        let effective = apply_data_bindings(&spec, &store);

        assert_eq!(
            effective.root.props.get("status"),
            Some(&Value::String("warning".into()))
        );
    }

    #[test]
    fn error_snapshot_can_preserve_cached_value() {
        let snapshot =
            DataSnapshot::error_with_cached_value("offline", DataValue::String("last".into()));

        assert_eq!(snapshot.status, DataStatus::Error);
        assert_eq!(snapshot.value, Some(DataValue::String("last".into())));
        assert_eq!(snapshot.error.as_deref(), Some("offline"));
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_runtime_reads_json_from_local_backend() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("local listener should bind");
        let address = listener.local_addr().expect("listener addr should resolve");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("request should arrive");
            let mut request = Vec::new();
            let mut buffer = [0_u8; 256];
            loop {
                let read = stream.read(&mut buffer).expect("request should read");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            let payload = b"{\"value\":\"ok\"}";
            let response = format!(
                "HTTP/1.0 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                payload.len(),
                std::str::from_utf8(payload).expect("payload is utf8")
            );
            stream
                .write_all(response.as_bytes())
                .expect("response should write");
            stream.flush().expect("response should flush");
        });

        let source = HttpSourceSpec {
            id: "ops".into(),
            url: format!("http://{address}/status"),
            method: HttpMethod::Get,
            headers: BTreeMap::new(),
            body: None,
            timeout_ms: Some(1_000),
            refresh_ms: None,
            retry_count: 0,
            retry_backoff_ms: 0,
        };

        let value = execute_http_source(&source).expect("local JSON should load");
        let DataValue::Object(object) = value else {
            panic!("response should be a JSON object");
        };
        assert_eq!(object.get("value"), Some(&DataValue::String("ok".into())));
        server.join().expect("server should finish");
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_runtime_reports_non_success_status() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("local listener should bind");
        let address = listener.local_addr().expect("listener addr should resolve");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("request should arrive");
            let mut request = Vec::new();
            let mut buffer = [0_u8; 256];
            loop {
                let read = stream.read(&mut buffer).expect("request should read");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }

            let payload = b"{\"error\":\"down\"}";
            let response = format!(
                "HTTP/1.0 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                payload.len(),
                std::str::from_utf8(payload).expect("payload is utf8")
            );
            stream
                .write_all(response.as_bytes())
                .expect("response should write");
            stream.flush().expect("response should flush");
        });

        let source = HttpSourceSpec {
            id: "ops".into(),
            url: format!("http://{address}/status"),
            method: HttpMethod::Get,
            headers: BTreeMap::new(),
            body: None,
            timeout_ms: Some(1_000),
            refresh_ms: None,
            retry_count: 0,
            retry_backoff_ms: 0,
        };

        let error = execute_http_source(&source).expect_err("500 should fail");

        assert!(error.to_string().contains("500"));
        server.join().expect("server should finish");
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_header_env_errors_do_not_include_secret_values() {
        let source = HttpSourceSpec {
            id: "secure".into(),
            url: "http://127.0.0.1:9".into(),
            method: HttpMethod::Get,
            headers: BTreeMap::from([(
                "Authorization".into(),
                HttpHeaderValue::Env {
                    env: "NEOTUI_TEST_MISSING_SECRET".into(),
                    prefix: Some("Bearer ".into()),
                    suffix: None,
                },
            )]),
            body: None,
            timeout_ms: Some(50),
            refresh_ms: None,
            retry_count: 0,
            retry_backoff_ms: 0,
        };

        let error = execute_http_source(&source).expect_err("missing env should fail");

        assert!(error.to_string().contains("NEOTUI_TEST_MISSING_SECRET"));
        assert!(!error.to_string().contains("Bearer "));
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_action_runtime_posts_and_requests_refresh() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::mpsc;

        let listener = TcpListener::bind("127.0.0.1:0").expect("local listener should bind");
        let address = listener.local_addr().expect("listener addr should resolve");
        let (request_tx, request_rx) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("request should arrive");
            let mut request = Vec::new();
            let mut buffer = [0_u8; 256];
            let mut expected_len = None;
            loop {
                let read = stream.read(&mut buffer).expect("request should read");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                if let Some(header_end) =
                    request.windows(4).position(|window| window == b"\r\n\r\n")
                {
                    if expected_len.is_none() {
                        let headers = String::from_utf8_lossy(&request[..header_end]);
                        expected_len = headers.lines().find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            if name.eq_ignore_ascii_case("content-length") {
                                value.trim().parse::<usize>().ok()
                            } else {
                                None
                            }
                        });
                    }
                    let body_start = header_end + 4;
                    let body_len = request.len().saturating_sub(body_start);
                    if body_len >= expected_len.unwrap_or(0) {
                        break;
                    }
                }
            }
            request_tx
                .send(String::from_utf8_lossy(&request).to_string())
                .expect("request should send to test");

            let payload = b"{\"ok\":true}";
            let response = format!(
                "HTTP/1.0 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                payload.len(),
                std::str::from_utf8(payload).expect("payload is utf8")
            );
            stream
                .write_all(response.as_bytes())
                .expect("response should write");
            stream.flush().expect("response should flush");
        });

        let action = ActionSpec {
            id: "refresh_now".into(),
            kind: ActionKind::Http,
            http: HttpSourceSpec {
                id: "refresh_now".into(),
                url: format!("http://{address}/ack"),
                method: HttpMethod::Post,
                headers: BTreeMap::new(),
                body: Some(HttpBody::Json(Value::Object(BTreeMap::from([(
                    "intent".into(),
                    Value::String("refresh".into()),
                )])))),
                timeout_ms: Some(1_000),
                refresh_ms: None,
                retry_count: 0,
                retry_backoff_ms: 0,
            },
            refresh_sources: vec!["ops".into()],
        };
        let mut runtime = HttpActionRuntime::new(&[action]);

        let loading = runtime
            .trigger("refresh_now")
            .expect("known action should trigger");
        assert_eq!(loading.status, ActionStatus::Loading);
        assert!(runtime.trigger("refresh_now").is_none());

        let ready = loop {
            let updates = runtime.poll();
            if let Some(update) = updates.into_iter().next() {
                break update;
            }
            std::thread::sleep(Duration::from_millis(5));
        };

        assert_eq!(ready.status, ActionStatus::Ready);
        assert_eq!(ready.refresh_sources, vec!["ops"]);
        assert_eq!(ready.error, None);
        let request = request_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("server should capture request");
        assert!(request.starts_with("POST /ack "));
        let (_, body) = request
            .split_once("\r\n\r\n")
            .expect("request should contain body separator");
        let json: serde_json::Value =
            serde_json::from_str(body).expect("request body should be JSON");
        assert_eq!(json["intent"], "refresh");
        server.join().expect("server should finish");
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_action_runtime_reports_error_without_refresh() {
        let action = ActionSpec {
            id: "refresh_now".into(),
            kind: ActionKind::Http,
            http: HttpSourceSpec {
                id: "refresh_now".into(),
                url: "http://127.0.0.1:9/ack".into(),
                method: HttpMethod::Post,
                headers: BTreeMap::new(),
                body: None,
                timeout_ms: Some(50),
                refresh_ms: None,
                retry_count: 0,
                retry_backoff_ms: 0,
            },
            refresh_sources: vec!["ops".into()],
        };
        let mut runtime = HttpActionRuntime::new(&[action]);

        let loading = runtime
            .trigger("refresh_now")
            .expect("known action should trigger");
        assert_eq!(loading.status, ActionStatus::Loading);

        let failed = loop {
            let updates = runtime.poll();
            if let Some(update) = updates.into_iter().next() {
                break update;
            }
            std::thread::sleep(Duration::from_millis(5));
        };

        assert_eq!(failed.status, ActionStatus::Error);
        assert!(failed.error.is_some());
        assert!(failed.refresh_sources.is_empty());
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_action_runtime_retries_before_success() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        let listener = TcpListener::bind("127.0.0.1:0").expect("local listener should bind");
        let address = listener.local_addr().expect("listener addr should resolve");
        let attempts = Arc::new(AtomicUsize::new(0));
        let server_attempts = Arc::clone(&attempts);
        let server = std::thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().expect("request should arrive");
                let mut request = Vec::new();
                let mut buffer = [0_u8; 256];
                loop {
                    let read = stream.read(&mut buffer).expect("request should read");
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..read]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }

                let attempt = server_attempts.fetch_add(1, Ordering::SeqCst);
                let (status, payload) = if attempt == 0 {
                    ("500 Internal Server Error", "{\"ok\":false}")
                } else {
                    ("200 OK", "{\"ok\":true}")
                };
                let response = format!(
                    "HTTP/1.0 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    payload.len(),
                    payload
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("response should write");
                stream.flush().expect("response should flush");
            }
        });

        let action = ActionSpec {
            id: "refresh_now".into(),
            kind: ActionKind::Http,
            http: HttpSourceSpec {
                id: "refresh_now".into(),
                url: format!("http://{address}/ack"),
                method: HttpMethod::Get,
                headers: BTreeMap::new(),
                body: None,
                timeout_ms: Some(1_000),
                refresh_ms: None,
                retry_count: 1,
                retry_backoff_ms: 1,
            },
            refresh_sources: vec!["ops".into()],
        };
        let mut runtime = HttpActionRuntime::new(&[action]);

        let _ = runtime.trigger("refresh_now");
        let ready = loop {
            let updates = runtime.poll();
            if let Some(update) = updates.into_iter().next() {
                break update;
            }
            std::thread::sleep(Duration::from_millis(5));
        };

        assert_eq!(ready.status, ActionStatus::Ready);
        assert_eq!(ready.refresh_sources, vec!["ops"]);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        server.join().expect("server should finish");
    }
}
