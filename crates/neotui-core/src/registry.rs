// Component registry
// Converts validated DSL component specs into NeoTUI runtime component trees

use std::fmt;

use crate::component::{ComponentNode, ComponentTree};
use crate::dsl::{AppSpec, ComponentSpec, Value};
use crate::render::TextAlign;
use crate::widgets::{Divider, DividerOrientation, Label, Panel, Spacer, Stack};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ComponentRegistry;

impl ComponentRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn build_tree(&self, spec: &AppSpec) -> Result<ComponentTree, RegistryError> {
        let root = self.build_node(&spec.root, "root")?;
        Ok(ComponentTree::new(root))
    }

    pub fn build_node(
        &self,
        spec: &ComponentSpec,
        path: &str,
    ) -> Result<ComponentNode, RegistryError> {
        let component = self.instantiate_component(spec, path)?;
        let children = spec
            .children
            .iter()
            .enumerate()
            .map(|(index, child)| self.build_node(child, &format!("{path}.children[{index}]")))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ComponentNode::new(component).with_children(children))
    }

    fn instantiate_component(
        &self,
        spec: &ComponentSpec,
        path: &str,
    ) -> Result<Box<dyn crate::component::Component>, RegistryError> {
        let id = component_id_for(spec, path);

        match spec.kind.as_str() {
            "Label" => {
                let text = required_string(spec, path, "text")?;
                let align = optional_align(spec, path)?;
                Ok(Box::new(Label::new(id, text).with_align(align)))
            }
            "Panel" => {
                let mut panel = Panel::new(id);

                if let Some(title) = optional_string(spec, path, "title")? {
                    panel = panel.with_title(title);
                }

                Ok(Box::new(panel))
            }
            "Divider" => {
                let mut divider = Divider::new(id);

                if let Some(orientation) = optional_orientation(spec, path)? {
                    divider = divider.with_orientation(orientation);
                }

                if let Some(symbol) = optional_char(spec, path, "symbol")? {
                    divider = divider.with_symbol(symbol);
                }

                Ok(Box::new(divider))
            }
            "Spacer" => Ok(Box::new(Spacer::new(id))),
            "VBox" => {
                let gap = optional_u16(spec, path, "gap")?.unwrap_or(0);
                Ok(Box::new(Stack::vertical(id).with_gap(gap)))
            }
            "HBox" => {
                let gap = optional_u16(spec, path, "gap")?.unwrap_or(0);
                Ok(Box::new(Stack::horizontal(id).with_gap(gap)))
            }
            "TextBlock" | "Button" | "List" | "Graph" => {
                Err(RegistryError::UnimplementedComponent {
                    path: path.into(),
                    kind: spec.kind.clone(),
                })
            }
            other => Err(RegistryError::UnknownComponent {
                path: path.into(),
                kind: other.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    UnknownComponent {
        path: String,
        kind: String,
    },
    UnimplementedComponent {
        path: String,
        kind: String,
    },
    MissingProperty {
        path: String,
        property: String,
    },
    InvalidPropertyType {
        path: String,
        property: String,
        expected: String,
    },
    InvalidPropertyValue {
        path: String,
        property: String,
        message: String,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownComponent { path, kind } => {
                write!(f, "{path}: unknown component kind `{kind}`")
            }
            Self::UnimplementedComponent { path, kind } => {
                write!(
                    f,
                    "{path}: component `{kind}` is known but not implemented yet"
                )
            }
            Self::MissingProperty { path, property } => {
                write!(
                    f,
                    "{path}.props.{property}: missing required property `{property}`"
                )
            }
            Self::InvalidPropertyType {
                path,
                property,
                expected,
            } => write!(
                f,
                "{path}.props.{property}: property `{property}` must be {expected}"
            ),
            Self::InvalidPropertyValue {
                path,
                property,
                message,
            } => write!(f, "{path}.props.{property}: {message}"),
        }
    }
}

impl std::error::Error for RegistryError {}

fn component_id_for(spec: &ComponentSpec, path: &str) -> String {
    spec.id.clone().unwrap_or_else(|| path.replace('.', "-"))
}

fn required_string(
    spec: &ComponentSpec,
    path: &str,
    property: &str,
) -> Result<String, RegistryError> {
    match spec.props.get(property) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: property.into(),
            expected: "a string".into(),
        }),
        None => Err(RegistryError::MissingProperty {
            path: path.into(),
            property: property.into(),
        }),
    }
}

fn optional_string(
    spec: &ComponentSpec,
    path: &str,
    property: &str,
) -> Result<Option<String>, RegistryError> {
    match spec.props.get(property) {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: property.into(),
            expected: "a string".into(),
        }),
        None => Ok(None),
    }
}

fn optional_char(
    spec: &ComponentSpec,
    path: &str,
    property: &str,
) -> Result<Option<char>, RegistryError> {
    let Some(value) = optional_string(spec, path, property)? else {
        return Ok(None);
    };

    let mut chars = value.chars();
    let Some(symbol) = chars.next() else {
        return Err(RegistryError::InvalidPropertyValue {
            path: path.into(),
            property: property.into(),
            message: format!("property `{property}` must not be empty"),
        });
    };

    if chars.next().is_some() {
        return Err(RegistryError::InvalidPropertyValue {
            path: path.into(),
            property: property.into(),
            message: format!("property `{property}` must be a single character"),
        });
    }

    Ok(Some(symbol))
}

fn optional_align(spec: &ComponentSpec, path: &str) -> Result<TextAlign, RegistryError> {
    match spec.props.get("align") {
        Some(Value::String(value)) => match value.as_str() {
            "left" => Ok(TextAlign::Left),
            "center" => Ok(TextAlign::Center),
            "right" => Ok(TextAlign::Right),
            _ => Err(RegistryError::InvalidPropertyValue {
                path: path.into(),
                property: "align".into(),
                message: format!(
                    "property `align` must be one of: left, center, right (got `{value}`)"
                ),
            }),
        },
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: "align".into(),
            expected: "a string".into(),
        }),
        None => Ok(TextAlign::Left),
    }
}

fn optional_u16(
    spec: &ComponentSpec,
    path: &str,
    property: &str,
) -> Result<Option<u16>, RegistryError> {
    match spec.props.get(property) {
        Some(Value::Integer(value)) => {
            let Ok(value) = u16::try_from(*value) else {
                return Err(RegistryError::InvalidPropertyValue {
                    path: path.into(),
                    property: property.into(),
                    message: format!("property `{property}` must be a non-negative integer"),
                });
            };

            Ok(Some(value))
        }
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: property.into(),
            expected: "a non-negative integer".into(),
        }),
        None => Ok(None),
    }
}

fn optional_orientation(
    spec: &ComponentSpec,
    path: &str,
) -> Result<Option<DividerOrientation>, RegistryError> {
    match spec.props.get("orientation") {
        Some(Value::String(value)) => match value.as_str() {
            "horizontal" => Ok(Some(DividerOrientation::Horizontal)),
            "vertical" => Ok(Some(DividerOrientation::Vertical)),
            _ => Err(RegistryError::InvalidPropertyValue {
                path: path.into(),
                property: "orientation".into(),
                message: format!(
                    "property `orientation` must be one of: horizontal, vertical (got `{value}`)"
                ),
            }),
        },
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: "orientation".into(),
            expected: "a string".into(),
        }),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn registry_builds_tree_for_supported_widgets() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: Some("minimal".into()),
            root: ComponentSpec {
                kind: "Panel".into(),
                id: Some("root-panel".into()),
                props: BTreeMap::from([("title".into(), Value::String("Stats".into()))]),
                children: vec![
                    ComponentSpec {
                        kind: "Label".into(),
                        id: Some("headline".into()),
                        props: BTreeMap::from([
                            ("text".into(), Value::String("Hello".into())),
                            ("align".into(), Value::String("center".into())),
                        ]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "Divider".into(),
                        id: None,
                        props: BTreeMap::from([
                            ("orientation".into(), Value::String("horizontal".into())),
                            ("symbol".into(), Value::String("=".into())),
                        ]),
                        children: Vec::new(),
                    },
                ],
            },
        };

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("supported widgets should instantiate");

        assert_eq!(
            tree.ids_depth_first()
                .into_iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
            vec!["root-panel", "headline", "root-children[1]"]
        );
    }

    #[test]
    fn registry_builds_tree_for_stack_containers() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            root: ComponentSpec {
                kind: "VBox".into(),
                id: Some("layout".into()),
                props: BTreeMap::from([("gap".into(), Value::Integer(1))]),
                children: vec![ComponentSpec {
                    kind: "HBox".into(),
                    id: Some("row".into()),
                    props: BTreeMap::from([("gap".into(), Value::Integer(2))]),
                    children: vec![
                        ComponentSpec {
                            kind: "Label".into(),
                            id: Some("left".into()),
                            props: BTreeMap::from([("text".into(), Value::String("Alpha".into()))]),
                            children: Vec::new(),
                        },
                        ComponentSpec {
                            kind: "Label".into(),
                            id: Some("right".into()),
                            props: BTreeMap::from([("text".into(), Value::String("Beta".into()))]),
                            children: Vec::new(),
                        },
                    ],
                }],
            },
        };

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("stack containers should instantiate");

        assert_eq!(
            tree.ids_depth_first()
                .into_iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
            vec!["layout", "row", "left", "right"]
        );
    }

    #[test]
    fn registry_rejects_negative_gap() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            root: ComponentSpec {
                kind: "VBox".into(),
                id: None,
                props: BTreeMap::from([("gap".into(), Value::Integer(-1))]),
                children: Vec::new(),
            },
        };

        let error = ComponentRegistry::new()
            .build_tree(&spec)
            .expect_err("negative gap should be rejected");

        assert_eq!(
            error.to_string(),
            "root.props.gap: property `gap` must be a non-negative integer"
        );
    }

    #[test]
    fn registry_rejects_known_but_unimplemented_component() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            root: ComponentSpec {
                kind: "Button".into(),
                id: None,
                props: BTreeMap::new(),
                children: Vec::new(),
            },
        };

        let error = ComponentRegistry::new()
            .build_tree(&spec)
            .expect_err("button should not instantiate yet");

        assert_eq!(
            error,
            RegistryError::UnimplementedComponent {
                path: "root".into(),
                kind: "Button".into(),
            }
        );
    }

    #[test]
    fn registry_rejects_invalid_symbol_shape() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            root: ComponentSpec {
                kind: "Divider".into(),
                id: None,
                props: BTreeMap::from([("symbol".into(), Value::String("==".into()))]),
                children: Vec::new(),
            },
        };

        let error = ComponentRegistry::new()
            .build_tree(&spec)
            .expect_err("divider symbol should be a single character");

        assert_eq!(
            error.to_string(),
            "root.props.symbol: property `symbol` must be a single character"
        );
    }

    #[test]
    fn registry_generates_stable_id_when_missing() {
        let spec = ComponentSpec {
            kind: "Spacer".into(),
            id: None,
            props: BTreeMap::new(),
            children: Vec::new(),
        };

        let node = ComponentRegistry::new()
            .build_node(&spec, "root.children[2]")
            .expect("spacer should instantiate");

        assert_eq!(node.id().0, "root-children[2]");
    }

    #[test]
    fn registry_builds_tree_from_dashboard_example() {
        let input = std::fs::read_to_string("examples/dashboard.toml")
            .expect("dashboard example should exist");
        let spec = AppSpec::from_toml_str(&input).expect("dashboard example should parse");

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("dashboard example should instantiate");

        assert_eq!(
            tree.ids_depth_first()
                .into_iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
            vec!["dashboard", "headline", "separator", "summary"]
        );
    }
}
