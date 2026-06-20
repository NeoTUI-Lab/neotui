// Declarative form state for user input intent

use crate::data::DataBinding;
use crate::dsl::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormSpec {
    pub id: String,
    pub fields: Vec<FormFieldSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldSpec {
    pub id: String,
    pub kind: FormFieldKind,
    pub initial: Option<Value>,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormFieldKind {
    #[default]
    Text,
}

impl std::str::FromStr for FormFieldKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "text" => Ok(Self::Text),
            _ => Err(format!("form field kind must be `text` (got `{value}`)")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormStore {
    values: HashMap<String, HashMap<String, Value>>,
}

impl FormStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(
        &mut self,
        form_id: impl Into<String>,
        field_id: impl Into<String>,
        value: Value,
    ) -> bool {
        let form_id = form_id.into();
        let field_id = field_id.into();
        let fields = self.values.entry(form_id).or_default();
        if fields.get(&field_id) == Some(&value) {
            return false;
        }
        fields.insert(field_id, value);
        true
    }

    pub fn get(&self, form_id: &str, field_id: &str) -> Option<&Value> {
        self.values.get(form_id)?.get(field_id)
    }

    pub fn resolve_binding(&self, binding: &DataBinding) -> Option<Value> {
        if binding.source != "$forms" {
            return None;
        }
        let form_id = binding.path.first()?;
        let field_id = binding.path.get(1)?;
        if binding.path.len() != 2 {
            return None;
        }
        self.get(form_id, field_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_store_resolves_declared_binding_paths() {
        let mut store = FormStore::new();
        let _ = store.set("incident", "summary", Value::String("Disk full".into()));

        let binding =
            DataBinding::parse("$forms.incident.summary").expect("form binding should parse");

        assert_eq!(
            store.resolve_binding(&binding),
            Some(Value::String("Disk full".into()))
        );
    }

    #[test]
    fn form_store_rejects_nested_field_binding_paths() {
        let mut store = FormStore::new();
        let _ = store.set("incident", "summary", Value::String("Disk full".into()));

        let binding =
            DataBinding::parse("$forms.incident.summary.extra").expect("form binding should parse");

        assert_eq!(store.resolve_binding(&binding), None);
    }
}
