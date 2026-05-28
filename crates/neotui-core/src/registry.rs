// Component registry
// Converts validated DSL component specs into NeoTUI runtime component trees

use std::fmt;

use crate::component::{ComponentNode, ComponentTree, LayoutHints};
use crate::dsl::{AppSpec, ComponentSpec, Value};
use crate::render::TextAlign;
use crate::theme::Theme;
use crate::widgets::{
    BigMetric, Button, Divider, DividerOrientation, Gauge, Graph, KeyValueRow, Knob, Label, List,
    Metric, Panel, PanelChrome, PanelDensity, PanelVariant, Spacer, Sparkline, Stack, StackAlign,
    StackJustify, StatusStrip, Table, TableColumn, TextBlock,
};
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ComponentRegistry;

impl ComponentRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn build_tree(&self, spec: &AppSpec) -> Result<ComponentTree, RegistryError> {
        let theme = theme_for_spec(spec);
        debug!(
            target: "neotui::registry",
            root_kind = spec.root.kind.as_str(),
            theme = theme.name().unwrap_or("baseline"),
            "building component tree"
        );
        let root = self.build_node_with_theme(&spec.root, "root", &theme)?;
        Ok(ComponentTree::new(root))
    }

    pub fn build_node(
        &self,
        spec: &ComponentSpec,
        path: &str,
    ) -> Result<ComponentNode, RegistryError> {
        self.build_node_with_theme(spec, path, &Theme::baseline())
    }

    fn build_node_with_theme(
        &self,
        spec: &ComponentSpec,
        path: &str,
        theme: &Theme,
    ) -> Result<ComponentNode, RegistryError> {
        debug!(
            target: "neotui::registry",
            path,
            kind = spec.kind.as_str(),
            child_count = spec.children.len(),
            "building component node"
        );
        let component = self.instantiate_component(spec, path, theme)?;
        let layout_hints = layout_hints_from_spec(spec, path)?;
        let children = spec
            .children
            .iter()
            .enumerate()
            .map(|(index, child)| {
                self.build_node_with_theme(child, &format!("{path}.children[{index}]"), theme)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ComponentNode::new(component)
            .with_layout_hints(layout_hints)
            .with_children(children))
    }

    fn instantiate_component(
        &self,
        spec: &ComponentSpec,
        path: &str,
        theme: &Theme,
    ) -> Result<Box<dyn crate::component::Component>, RegistryError> {
        let id = component_id_for(spec, path);
        debug!(
            target: "neotui::registry",
            path,
            kind = spec.kind.as_str(),
            component_id = id.as_str(),
            "instantiating component"
        );

        match spec.kind.as_str() {
            "Label" => {
                let text = required_string(spec, path, "text")?;
                let align = optional_align(spec, path)?;
                Ok(Box::new(
                    Label::new(id, text)
                        .with_align(align)
                        .with_style(theme.resolve_style("label.default")),
                ))
            }
            "TextBlock" => {
                let text = required_string(spec, path, "text")?;
                Ok(Box::new(
                    TextBlock::new(id, text).with_style(theme.resolve_style("text_block.default")),
                ))
            }
            "Button" => {
                let text = required_string(spec, path, "text")?;
                let variant = optional_string(spec, path, "variant")?;
                let (style, focused_style) = if let Some(ref var) = variant {
                    (
                        theme.resolve_style(&format!("button.{}", var)),
                        theme.resolve_style(&format!("button.{}.focused", var)),
                    )
                } else {
                    (
                        theme.resolve_style("button.default"),
                        theme.resolve_style("button.focused"),
                    )
                };
                let mut button = Button::new(id, text)
                    .with_style(style)
                    .with_focused_style(focused_style);
                if let Some(var) = variant {
                    button = button.with_variant(var);
                }
                Ok(Box::new(button))
            }
            "List" => {
                let items = required_string_array(spec, path, "items")?;
                let mut list = List::new(id, items)
                    .with_style(theme.resolve_style("list.default"))
                    .with_selected_style(theme.resolve_style("list.selected"));
                if let Some(title) = optional_string(spec, path, "title")? {
                    list = list.with_title(title);
                }
                Ok(Box::new(list))
            }
            "Graph" => {
                let values = required_number_array(spec, path, "values")?;
                let mut graph =
                    Graph::new(id, values).with_style(theme.resolve_style("graph.default"));
                if let Some(title) = optional_string(spec, path, "title")? {
                    graph = graph.with_title(title);
                }
                Ok(Box::new(graph))
            }
            "Table" => {
                let columns = required_table_columns(spec, path)?;
                let rows = required_table_rows(spec, path, &columns)?;
                Ok(Box::new(
                    Table::new(id, columns, rows)
                        .with_style(theme.resolve_style("table.row"))
                        .with_header_style(theme.resolve_style("table.header"))
                        .with_selected_style(theme.resolve_style("table.selected")),
                ))
            }
            "Metric" => {
                let title = required_string(spec, path, "title")?;
                let value = required_string(spec, path, "value")?;
                let status =
                    optional_string(spec, path, "status")?.unwrap_or_else(|| "normal".to_string());
                let status_tok = format!("status.{}", status.to_lowercase());
                let mut metric = Metric::new(id, title, value)
                    .with_style(theme.resolve_style("metric.default"))
                    .with_status(status)
                    .with_status_style(theme.resolve_style(&status_tok));
                if let Some(delta) = optional_string(spec, path, "delta")? {
                    metric = metric.with_delta(delta);
                }
                Ok(Box::new(metric))
            }
            "Gauge" => {
                let value = required_double(spec, path, "value")?;
                let min = optional_double(spec, path, "min")?.unwrap_or(0.0);
                let max = optional_double(spec, path, "max")?.unwrap_or(100.0);
                let mut gauge = Gauge::new(id, value)
                    .with_min_max(min, max)
                    .with_style(theme.resolve_style("gauge.track"))
                    .with_filled_style(theme.resolve_style("gauge.filled"));
                if let Some(title) = optional_string(spec, path, "title")? {
                    gauge = gauge.with_title(title);
                }
                if let Some(orientation) = optional_orientation(spec, path)? {
                    gauge = gauge.with_orientation(orientation);
                }
                if let Some(fs) = optional_string(spec, path, "fill_style")? {
                    let fill = match fs.as_str() {
                        "gradient" => crate::widgets::gauge::GaugeFillStyle::Gradient,
                        "block" => crate::widgets::gauge::GaugeFillStyle::Block,
                        _ => crate::widgets::gauge::GaugeFillStyle::Solid,
                    };
                    gauge = gauge.with_fill_style(fill);
                }
                Ok(Box::new(gauge))
            }
            "Sparkline" => {
                let values = required_number_array(spec, path, "values")?;
                let mut spark =
                    Sparkline::new(id, values).with_style(theme.resolve_style("sparkline.default"));
                if let Some(title) = optional_string(spec, path, "title")? {
                    spark = spark.with_title(title);
                }
                Ok(Box::new(spark))
            }
            "KeyValueRow" => {
                let key = required_string(spec, path, "key")?;
                let value = required_string(spec, path, "value")?;
                let mut row = KeyValueRow::new(id, key, value)
                    .with_style(theme.resolve_style("key_value_row.default"));
                if let Some(conn_str) = optional_string(spec, path, "connector")? {
                    if let Some(connector) = conn_str.chars().next() {
                        row = row.with_connector(connector);
                    }
                }
                Ok(Box::new(row))
            }
            "StatusStrip" => {
                let text = required_string(spec, path, "text")?;
                let status =
                    optional_string(spec, path, "status")?.unwrap_or_else(|| "normal".to_string());
                let status_tok = format!("status.{}.tag", status.to_lowercase());
                let mut strip = StatusStrip::new(id, text)
                    .with_style(theme.resolve_style("status_strip.default"))
                    .with_status(status)
                    .with_status_style(theme.resolve_style(&status_tok));
                if let Some(fill) = optional_string(spec, path, "fill")? {
                    strip = strip.with_fill(fill);
                }
                Ok(Box::new(strip))
            }
            "BigMetric" => {
                let value = required_string(spec, path, "value")?;
                let mut widget =
                    BigMetric::new(id, value).with_style(theme.resolve_style("metric.default"));
                if let Some(title) = optional_string(spec, path, "title")? {
                    widget = widget.with_title(title);
                }
                if let Some(unit) = optional_string(spec, path, "unit")? {
                    widget = widget.with_unit(unit);
                }
                // font prop takes priority over legacy scale
                if let Some(font_str) = optional_string(spec, path, "font")? {
                    let font = match font_str.as_str() {
                        "large" => crate::widgets::big_metric::BigFont::Large,
                        "hero" => crate::widgets::big_metric::BigFont::Hero,
                        _ => crate::widgets::big_metric::BigFont::Compact,
                    };
                    widget = widget.with_font(font);
                } else if let Some(scale) = optional_u16(spec, path, "scale")? {
                    widget = widget.with_scale(scale.clamp(1, 3) as u8);
                }
                Ok(Box::new(widget))
            }
            "Knob" => {
                let value = required_double(spec, path, "value")?;
                let min = optional_double(spec, path, "min")?.unwrap_or(0.0);
                let max = optional_double(spec, path, "max")?.unwrap_or(100.0);
                let style = theme.resolve_style("knob.default");
                let mut knob = Knob::new(id, value)
                    .with_min_max(min, max)
                    .with_style(style);
                if let Some(title) = optional_string(spec, path, "title")? {
                    knob = knob.with_title(title);
                }
                Ok(Box::new(knob))
            }
            "Panel" => {
                let variant_name =
                    optional_string(spec, path, "variant")?.unwrap_or_else(|| "framed".into());
                let variant = panel_variant(&variant_name);
                let chrome_name =
                    optional_string(spec, path, "chrome")?.unwrap_or_else(|| "framed".into());
                let chrome = panel_chrome(&chrome_name);
                let density_name =
                    optional_string(spec, path, "density")?.unwrap_or_else(|| "normal".into());
                let density = panel_density(&density_name);
                let border_token = panel_border_token(&variant_name);
                let surface_token = panel_surface_token(&variant_name);
                let mut panel = Panel::new(id)
                    .with_variant(variant)
                    .with_chrome(chrome)
                    .with_density(density)
                    .with_style(theme.resolve_style(border_token))
                    .with_surface_style(theme.resolve_style(surface_token));

                if let Some(title) = optional_string(spec, path, "title")? {
                    panel = panel.with_title(title);
                }

                if let Some(border_style_name) = optional_string(spec, path, "border_style")? {
                    panel = panel.with_border_style_name(border_style_name.clone());
                    let border_glyphs = match border_style_name.as_str() {
                        "double" => crate::render::BorderStyle::double(),
                        "rounded" => crate::render::BorderStyle::rounded(),
                        "hex" => crate::render::BorderStyle::hex(),
                        "angular" => crate::render::BorderStyle::angular(),
                        _ => crate::render::BorderStyle::single(),
                    };
                    panel = panel.with_border(border_glyphs);
                } else {
                    panel = panel.with_border(default_panel_border(variant, chrome));
                }

                if let Some(grid) = optional_bool(spec, path, "grid")? {
                    panel = panel.with_grid(grid);
                    if grid {
                        panel = panel.with_grid_style(theme.resolve_style("grid.dot"));
                    }
                }

                if let Some(controls) = optional_bool(spec, path, "controls")? {
                    panel = panel.with_controls(controls);
                }

                if let Some(ts) = optional_string(spec, path, "title_style")? {
                    let title_style = match ts.as_str() {
                        "chevron" => crate::widgets::panel::TitleStyle::Chevron,
                        "bracket" => crate::widgets::panel::TitleStyle::Bracket,
                        "arrow" => crate::widgets::panel::TitleStyle::Arrow,
                        _ => crate::widgets::panel::TitleStyle::Plain,
                    };
                    panel = panel.with_title_style(title_style);
                }

                if let Some(fl) = optional_string(spec, path, "footer_left")? {
                    panel = panel.with_footer_left(fl);
                }
                if let Some(fr) = optional_string(spec, path, "footer_right")? {
                    panel = panel.with_footer_right(fr);
                }

                Ok(Box::new(panel))
            }
            "Divider" => {
                let mut divider =
                    Divider::new(id).with_style(theme.resolve_style("divider.default"));

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
                let align = optional_stack_align(spec, path)?.unwrap_or_default();
                let justify = optional_stack_justify(spec, path)?.unwrap_or_default();
                Ok(Box::new(
                    Stack::vertical(id)
                        .with_gap(gap)
                        .with_align(align)
                        .with_justify(justify),
                ))
            }
            "HBox" => {
                let gap = optional_u16(spec, path, "gap")?.unwrap_or(0);
                let align = optional_stack_align(spec, path)?.unwrap_or_default();
                let justify = optional_stack_justify(spec, path)?.unwrap_or_default();
                Ok(Box::new(
                    Stack::horizontal(id)
                        .with_gap(gap)
                        .with_align(align)
                        .with_justify(justify),
                ))
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

fn theme_for_spec(spec: &AppSpec) -> Theme {
    spec.theme
        .as_deref()
        .and_then(Theme::preset)
        .unwrap_or_else(Theme::baseline)
}

fn component_id_for(spec: &ComponentSpec, path: &str) -> String {
    spec.id.clone().unwrap_or_else(|| path.replace('.', "-"))
}

fn panel_variant(value: &str) -> PanelVariant {
    match value {
        "plain" => PanelVariant::Plain,
        "data" | "info" | "success" | "warning" => PanelVariant::Data,
        "alert" | "danger" => PanelVariant::Alert,
        "hero" => PanelVariant::Hero,
        _ => PanelVariant::Framed,
    }
}

fn panel_density(value: &str) -> PanelDensity {
    match value {
        "compact" => PanelDensity::Compact,
        "spacious" => PanelDensity::Spacious,
        _ => PanelDensity::Normal,
    }
}

fn panel_chrome(value: &str) -> PanelChrome {
    match value {
        "minimal" => PanelChrome::Minimal,
        "technical" => PanelChrome::Technical,
        "cinematic" => PanelChrome::Cinematic,
        _ => PanelChrome::Framed,
    }
}

fn panel_border_token(variant: &str) -> &'static str {
    match variant {
        "plain" => "panel.border.subtle",
        "data" | "info" => "panel.border.data",
        "alert" | "danger" => "panel.border.alert",
        "warning" => "panel.border.warning",
        "success" => "panel.border.success",
        "hero" => "panel.border.hero",
        _ => "panel.border",
    }
}

fn panel_surface_token(variant: &str) -> &'static str {
    match variant {
        "plain" => "panel.surface.plain",
        "data" | "info" => "panel.surface.data",
        "alert" | "danger" => "panel.surface.alert",
        "warning" => "panel.surface.warning",
        "success" => "panel.surface.success",
        "hero" => "panel.surface.hero",
        _ => "panel.surface",
    }
}

fn default_panel_border(variant: PanelVariant, chrome: PanelChrome) -> crate::render::BorderStyle {
    match (variant, chrome) {
        (PanelVariant::Plain, _) | (_, PanelChrome::Minimal) => {
            crate::render::BorderStyle::single()
        }
        (PanelVariant::Hero, _) => crate::render::BorderStyle::double(),
        (PanelVariant::Alert, _) => crate::render::BorderStyle::angular(),
        (_, PanelChrome::Technical) | (_, PanelChrome::Cinematic) => {
            crate::render::BorderStyle::hex()
        }
        (PanelVariant::Data, _) => crate::render::BorderStyle::rounded(),
        _ => crate::render::BorderStyle::single(),
    }
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

fn required_string_array(
    spec: &ComponentSpec,
    path: &str,
    property: &str,
) -> Result<Vec<String>, RegistryError> {
    match spec.props.get(property) {
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| match value {
                Value::String(value) => Ok(value.clone()),
                _ => Err(RegistryError::InvalidPropertyType {
                    path: path.into(),
                    property: property.into(),
                    expected: "an array of strings".into(),
                }),
            })
            .collect(),
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: property.into(),
            expected: "an array of strings".into(),
        }),
        None => Err(RegistryError::MissingProperty {
            path: path.into(),
            property: property.into(),
        }),
    }
}

fn required_number_array(
    spec: &ComponentSpec,
    path: &str,
    property: &str,
) -> Result<Vec<f64>, RegistryError> {
    match spec.props.get(property) {
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| match value {
                Value::Integer(value) => Ok(*value as f64),
                Value::Float(value) => {
                    value
                        .parse::<f64>()
                        .map_err(|_| RegistryError::InvalidPropertyValue {
                            path: path.into(),
                            property: property.into(),
                            message: format!("property `{property}` must contain valid numbers"),
                        })
                }
                _ => Err(RegistryError::InvalidPropertyType {
                    path: path.into(),
                    property: property.into(),
                    expected: "an array of numbers".into(),
                }),
            })
            .collect(),
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: property.into(),
            expected: "an array of numbers".into(),
        }),
        None => Err(RegistryError::MissingProperty {
            path: path.into(),
            property: property.into(),
        }),
    }
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

fn align_from_value(value: &Value, path: &str, property: &str) -> Result<TextAlign, RegistryError> {
    match value {
        Value::String(value) => match value.as_str() {
            "left" => Ok(TextAlign::Left),
            "center" => Ok(TextAlign::Center),
            "right" => Ok(TextAlign::Right),
            _ => Err(RegistryError::InvalidPropertyValue {
                path: path.into(),
                property: property.into(),
                message: format!(
                    "property `{property}` must be one of: left, center, right (got `{value}`)"
                ),
            }),
        },
        _ => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: property.into(),
            expected: "a string".into(),
        }),
    }
}

fn required_table_columns(
    spec: &ComponentSpec,
    path: &str,
) -> Result<Vec<TableColumn>, RegistryError> {
    let values = match spec.props.get("columns") {
        Some(Value::Array(values)) => values,
        Some(_) => {
            return Err(RegistryError::InvalidPropertyType {
                path: path.into(),
                property: "columns".into(),
                expected: "an array of column objects".into(),
            });
        }
        None => {
            return Err(RegistryError::MissingProperty {
                path: path.into(),
                property: "columns".into(),
            });
        }
    };

    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let column_path = format!("{path}.props.columns[{index}]");
            let Value::Object(column_object) = value else {
                return Err(RegistryError::InvalidPropertyType {
                    path: column_path,
                    property: "columns".into(),
                    expected: "a column object".into(),
                });
            };

            let key = required_object_string(column_object, &column_path, "key")?;
            let title = required_object_string(column_object, &column_path, "title")?;
            let width = required_object_u16(column_object, &column_path, "width")?;
            let mut column = TableColumn::new(key, title, width);

            if let Some(align_value) = column_object.get("align") {
                let align = align_from_value(align_value, &column_path, "align")?;
                column = column.with_align(align);
            }

            Ok(column)
        })
        .collect()
}

fn required_table_rows(
    spec: &ComponentSpec,
    path: &str,
    columns: &[TableColumn],
) -> Result<Vec<Vec<String>>, RegistryError> {
    let values = match spec.props.get("rows") {
        Some(Value::Array(values)) => values,
        Some(_) => {
            return Err(RegistryError::InvalidPropertyType {
                path: path.into(),
                property: "rows".into(),
                expected: "an array of row objects".into(),
            });
        }
        None => {
            return Err(RegistryError::MissingProperty {
                path: path.into(),
                property: "rows".into(),
            });
        }
    };

    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let row_path = format!("{path}.props.rows[{index}]");
            let Value::Object(row) = value else {
                return Err(RegistryError::InvalidPropertyType {
                    path: row_path,
                    property: "rows".into(),
                    expected: "a row object".into(),
                });
            };

            Ok(columns
                .iter()
                .map(|column| {
                    row.get(column.key())
                        .map(table_cell_to_string)
                        .unwrap_or_default()
                })
                .collect())
        })
        .collect()
}

fn required_object_string(
    object: &std::collections::BTreeMap<String, Value>,
    path: &str,
    property: &str,
) -> Result<String, RegistryError> {
    match object.get(property) {
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

fn required_object_u16(
    object: &std::collections::BTreeMap<String, Value>,
    path: &str,
    property: &str,
) -> Result<u16, RegistryError> {
    match object.get(property) {
        Some(Value::Integer(value)) => {
            let Ok(value) = u16::try_from(*value) else {
                return Err(RegistryError::InvalidPropertyValue {
                    path: path.into(),
                    property: property.into(),
                    message: format!("property `{property}` must be greater than zero"),
                });
            };

            if value == 0 {
                return Err(RegistryError::InvalidPropertyValue {
                    path: path.into(),
                    property: property.into(),
                    message: format!("property `{property}` must be greater than zero"),
                });
            }

            Ok(value)
        }
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: property.into(),
            expected: "a positive integer".into(),
        }),
        None => Err(RegistryError::MissingProperty {
            path: path.into(),
            property: property.into(),
        }),
    }
}

fn table_cell_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Integer(value) => value.to_string(),
        Value::Float(value) => value.clone(),
        Value::String(value) => value.clone(),
        Value::Array(_) | Value::Object(_) => String::new(),
    }
}

fn required_double(spec: &ComponentSpec, path: &str, property: &str) -> Result<f64, RegistryError> {
    match spec.props.get(property) {
        Some(Value::Integer(value)) => Ok(*value as f64),
        Some(Value::Float(value)) => {
            value
                .parse::<f64>()
                .map_err(|_| RegistryError::InvalidPropertyValue {
                    path: path.into(),
                    property: property.into(),
                    message: format!("property `{property}` must be a valid number"),
                })
        }
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: property.into(),
            expected: "a number".into(),
        }),
        None => Err(RegistryError::MissingProperty {
            path: path.into(),
            property: property.into(),
        }),
    }
}

fn optional_double(
    spec: &ComponentSpec,
    path: &str,
    property: &str,
) -> Result<Option<f64>, RegistryError> {
    match spec.props.get(property) {
        Some(Value::Integer(value)) => Ok(Some(*value as f64)),
        Some(Value::Float(value)) => {
            value
                .parse::<f64>()
                .map(Some)
                .map_err(|_| RegistryError::InvalidPropertyValue {
                    path: path.into(),
                    property: property.into(),
                    message: format!("property `{property}` must be a valid number"),
                })
        }
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: property.into(),
            expected: "a number".into(),
        }),
        None => Ok(None),
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

fn optional_bool(
    spec: &ComponentSpec,
    path: &str,
    property: &str,
) -> Result<Option<bool>, RegistryError> {
    match spec.props.get(property) {
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: property.into(),
            expected: "a boolean".into(),
        }),
        None => Ok(None),
    }
}

fn optional_percent(
    spec: &ComponentSpec,
    path: &str,
    property: &str,
) -> Result<Option<u16>, RegistryError> {
    let Some(value) = optional_u16(spec, path, property)? else {
        return Ok(None);
    };

    if value > 100 {
        return Err(RegistryError::InvalidPropertyValue {
            path: path.into(),
            property: property.into(),
            message: format!("property `{property}` must be between 0 and 100"),
        });
    }

    Ok(Some(value))
}

fn optional_positive_u16(
    spec: &ComponentSpec,
    path: &str,
    property: &str,
) -> Result<Option<u16>, RegistryError> {
    let Some(value) = optional_u16(spec, path, property)? else {
        return Ok(None);
    };

    if value == 0 {
        return Err(RegistryError::InvalidPropertyValue {
            path: path.into(),
            property: property.into(),
            message: format!("property `{property}` must be greater than zero"),
        });
    }

    Ok(Some(value))
}

fn layout_hints_from_spec(spec: &ComponentSpec, path: &str) -> Result<LayoutHints, RegistryError> {
    Ok(LayoutHints {
        width: optional_u16(spec, path, "width")?,
        height: optional_u16(spec, path, "height")?,
        width_pct: optional_percent(spec, path, "width_pct")?,
        height_pct: optional_percent(spec, path, "height_pct")?,
        grow: optional_positive_u16(spec, path, "grow")?,
    })
}

fn optional_stack_align(
    spec: &ComponentSpec,
    path: &str,
) -> Result<Option<StackAlign>, RegistryError> {
    match spec.props.get("align") {
        Some(Value::String(value)) => match value.as_str() {
            "start" => Ok(Some(StackAlign::Start)),
            "center" => Ok(Some(StackAlign::Center)),
            "end" => Ok(Some(StackAlign::End)),
            "stretch" => Ok(Some(StackAlign::Stretch)),
            _ => Err(RegistryError::InvalidPropertyValue {
                path: path.into(),
                property: "align".into(),
                message: format!(
                    "property `align` must be one of: start, center, end, stretch (got `{value}`)"
                ),
            }),
        },
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: "align".into(),
            expected: "a string".into(),
        }),
        None => Ok(None),
    }
}

fn optional_stack_justify(
    spec: &ComponentSpec,
    path: &str,
) -> Result<Option<StackJustify>, RegistryError> {
    match spec.props.get("justify") {
        Some(Value::String(value)) => match value.as_str() {
            "start" => Ok(Some(StackJustify::Start)),
            "center" => Ok(Some(StackJustify::Center)),
            "end" => Ok(Some(StackJustify::End)),
            _ => Err(RegistryError::InvalidPropertyValue {
                path: path.into(),
                property: "justify".into(),
                message: format!(
                    "property `justify` must be one of: start, center, end (got `{value}`)"
                ),
            }),
        },
        Some(_) => Err(RegistryError::InvalidPropertyType {
            path: path.into(),
            property: "justify".into(),
            expected: "a string".into(),
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
    use std::path::PathBuf;

    use super::*;
    use crate::component::{EventContext, LayoutContext};
    use crate::event::{
        ComponentId, Event, EventResult, KeyCode, KeyEvent, KeyModifiers, ScrollDirection,
        ScrollEvent,
    };
    use crate::layout::Rect;
    use crate::render::{Color, ScreenBuffer};

    fn fixture_path(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    }

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
    fn registry_applies_redline_theme_to_current_widgets() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: Some("redline".into()),
            root: ComponentSpec {
                kind: "Label".into(),
                id: Some("alert".into()),
                props: BTreeMap::from([("text".into(), Value::String("REBOOT FAILED".into()))]),
                children: Vec::new(),
            },
        };
        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("redline themed widget should instantiate");
        let mut frame = ScreenBuffer::new(16, 1);
        let layout = tree.layout(&LayoutContext, Rect::new(0, 0, 16, 1));

        tree.render_with_layout(&layout, &mut frame);

        let style = frame
            .get(0, 0)
            .map(|cell| cell.style.clone())
            .expect("first label cell should render");
        assert_eq!(
            style.fg,
            Color::Rgb {
                r: 224,
                g: 232,
                b: 238,
            }
        );
        assert!(style.bold);
    }

    #[test]
    fn registry_applies_redline_variants_and_statuses() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: Some("redline".into()),
            root: ComponentSpec {
                kind: "VBox".into(),
                id: Some("box".into()),
                props: BTreeMap::new(),
                children: vec![
                    ComponentSpec {
                        kind: "Button".into(),
                        id: Some("btn".into()),
                        props: BTreeMap::from([
                            ("text".into(), Value::String("Purge".into())),
                            ("variant".into(), Value::String("danger".into())),
                        ]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "Panel".into(),
                        id: Some("pan".into()),
                        props: BTreeMap::from([(
                            "variant".into(),
                            Value::String("warning".into()),
                        )]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "Metric".into(),
                        id: Some("met".into()),
                        props: BTreeMap::from([
                            ("title".into(), Value::String("TEMP".into())),
                            ("value".into(), Value::String("98-16".into())),
                            ("status".into(), Value::String("warning".into())),
                        ]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "StatusStrip".into(),
                        id: Some("str".into()),
                        props: BTreeMap::from([
                            ("text".into(), Value::String("REBOOT".into())),
                            ("status".into(), Value::String("critical".into())),
                        ]),
                        children: Vec::new(),
                    },
                ],
            },
        };
        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("themed widgets with variants should instantiate");

        let layout = tree.layout(&LayoutContext, Rect::new(0, 0, 40, 20));
        let mut frame = ScreenBuffer::new(40, 20);
        tree.render_with_layout(&layout, &mut frame);

        // Button "Purge" (variant danger) should render with danger color (fg: Rgb { r: 255, g: 35, b: 48 })
        let btn_layout = layout.children.first().expect("button layout should exist");
        let btn_y = btn_layout.area.y + btn_layout.area.height.saturating_sub(1) / 2;
        let btn_cell = frame
            .get(btn_layout.area.x + btn_layout.area.width / 2, btn_y)
            .expect("button center cell should exist");
        assert_eq!(
            btn_cell.style.fg,
            Color::Rgb {
                r: 255,
                g: 35,
                b: 48
            }
        );

        // Panel (variant warning) border should render with warning color (fg: Rgb { r: 255, g: 117, b: 74 })
        let pan_layout = layout.children.get(1).expect("panel layout should exist");
        let pan_border_cell = frame
            .get(pan_layout.area.x, pan_layout.area.y)
            .expect("panel border corner cell should exist");
        assert_eq!(
            pan_border_cell.style.fg,
            Color::Rgb {
                r: 255,
                g: 117,
                b: 74
            }
        );

        // StatusStrip (status critical) tag block should render with danger bg (Color::Rgb { r: 255, g: 35, b: 48 })
        let str_layout = layout
            .children
            .get(3)
            .expect("status strip layout should exist");
        let str_tag_cell = frame
            .get(str_layout.area.x + 1, str_layout.area.y)
            .expect("status strip tag cell should exist");
        assert_eq!(
            str_tag_cell.style.bg,
            Color::Rgb {
                r: 255,
                g: 35,
                b: 48
            }
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
    fn registry_applies_stack_alignment_and_justify_props() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            root: ComponentSpec {
                kind: "VBox".into(),
                id: Some("layout".into()),
                props: BTreeMap::from([
                    ("align".into(), Value::String("center".into())),
                    ("justify".into(), Value::String("end".into())),
                ]),
                children: vec![ComponentSpec {
                    kind: "Label".into(),
                    id: Some("child".into()),
                    props: BTreeMap::from([
                        ("text".into(), Value::String("Hello".into())),
                        ("width".into(), Value::Integer(4)),
                        ("height".into(), Value::Integer(1)),
                    ]),
                    children: Vec::new(),
                }],
            },
        };

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("stack props should instantiate");

        let layout = tree.layout(
            &crate::component::LayoutContext,
            crate::layout::Rect::new(0, 0, 10, 4),
        );

        assert_eq!(
            layout.children[0].area,
            crate::layout::Rect::new(3, 3, 4, 1)
        );
    }

    #[test]
    fn registry_applies_layout_hints_to_children() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            root: ComponentSpec {
                kind: "HBox".into(),
                id: Some("layout".into()),
                props: BTreeMap::new(),
                children: vec![
                    ComponentSpec {
                        kind: "Label".into(),
                        id: Some("fixed".into()),
                        props: BTreeMap::from([
                            ("text".into(), Value::String("Fixed".into())),
                            ("width".into(), Value::Integer(6)),
                        ]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "Label".into(),
                        id: Some("flex".into()),
                        props: BTreeMap::from([
                            ("text".into(), Value::String("Flex".into())),
                            ("grow".into(), Value::Integer(2)),
                        ]),
                        children: Vec::new(),
                    },
                ],
            },
        };

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("layout hints should instantiate");

        let layout = tree.layout(
            &crate::component::LayoutContext,
            crate::layout::Rect::new(0, 0, 20, 3),
        );

        assert_eq!(layout.children[0].area.width, 6);
        assert_eq!(layout.children[1].area.width, 14);
    }

    #[test]
    fn registry_rejects_invalid_width_percent() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            root: ComponentSpec {
                kind: "Label".into(),
                id: None,
                props: BTreeMap::from([
                    ("text".into(), Value::String("Hello".into())),
                    ("width_pct".into(), Value::Integer(120)),
                ]),
                children: Vec::new(),
            },
        };

        let error = ComponentRegistry::new()
            .build_tree(&spec)
            .expect_err("width_pct above 100 should fail");

        assert_eq!(
            error.to_string(),
            "root.props.width_pct: property `width_pct` must be between 0 and 100"
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
    fn registry_builds_tree_for_new_leaf_widgets() {
        let spec = AppSpec {
            schema_version: "0.1".into(),
            theme: None,
            root: ComponentSpec {
                kind: "VBox".into(),
                id: Some("layout".into()),
                props: BTreeMap::new(),
                children: vec![
                    ComponentSpec {
                        kind: "Button".into(),
                        id: Some("deploy".into()),
                        props: BTreeMap::from([("text".into(), Value::String("Deploy".into()))]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "TextBlock".into(),
                        id: Some("notes".into()),
                        props: BTreeMap::from([(
                            "text".into(),
                            Value::String("alpha\nbeta".into()),
                        )]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "List".into(),
                        id: Some("services".into()),
                        props: BTreeMap::from([
                            (
                                "items".into(),
                                Value::Array(vec![
                                    Value::String("api".into()),
                                    Value::String("jobs".into()),
                                ]),
                            ),
                            ("title".into(), Value::String("Services".into())),
                        ]),
                        children: Vec::new(),
                    },
                    ComponentSpec {
                        kind: "Graph".into(),
                        id: Some("latency".into()),
                        props: BTreeMap::from([(
                            "values".into(),
                            Value::Array(vec![Value::Integer(1), Value::Float("2.5".into())]),
                        )]),
                        children: Vec::new(),
                    },
                ],
            },
        };

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("new widgets should instantiate");

        assert_eq!(
            tree.ids_depth_first()
                .into_iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
            vec!["layout", "deploy", "notes", "services", "latency"]
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
        let input = std::fs::read_to_string(fixture_path("examples/dashboard.toml"))
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

    #[test]
    fn registry_builds_tree_from_list_demo_example() {
        let input = std::fs::read_to_string(fixture_path("examples/list-demo.toml"))
            .expect("list demo should exist");
        let spec = AppSpec::from_toml_str(&input).expect("list demo should parse");

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("list demo should instantiate");

        assert_eq!(
            tree.ids_depth_first()
                .into_iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
            vec!["list-demo", "list-heading", "list-rule", "services"]
        );
    }

    #[test]
    fn registry_builds_tree_from_showcase_layout_example() {
        let input = std::fs::read_to_string(fixture_path("examples/showcase-layout.toml"))
            .expect("showcase layout example should exist");
        let spec = AppSpec::from_toml_str(&input).expect("showcase layout example should parse");

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("showcase layout example should instantiate");

        assert_eq!(
            tree.ids_depth_first()
                .into_iter()
                .map(|id| id.0)
                .collect::<Vec<_>>(),
            vec![
                "showcase",
                "content",
                "headline",
                "rule",
                "stats",
                "service-a",
                "service-b",
                "service-c",
                "footer",
            ]
        );
    }

    #[test]
    fn registry_builds_tree_from_rich_dashboard_example() {
        let input = std::fs::read_to_string(fixture_path("examples/rich-dashboard.toml"))
            .expect("rich dashboard example should exist");
        let spec = AppSpec::from_toml_str(&input).expect("rich dashboard example should parse");

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("rich dashboard example should instantiate");

        let ids = tree
            .ids_depth_first()
            .into_iter()
            .map(|id| id.0)
            .collect::<Vec<_>>();

        assert_eq!(ids.first().map(String::as_str), Some("rich-dashboard"));
        assert!(ids.iter().any(|id| id == "service-list"));
        assert!(ids.iter().any(|id| id == "throughput-graph"));
        assert!(ids.iter().any(|id| id == "operator-notes"));
        assert!(ids.iter().any(|id| id == "deploy-button"));
    }

    #[test]
    fn registry_builds_tree_from_redline_dashboard_example() {
        let input = std::fs::read_to_string(fixture_path("examples/redline-dashboard.toml"))
            .expect("redline dashboard example should exist");
        let spec = AppSpec::from_toml_str(&input).expect("redline dashboard example should parse");

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("redline dashboard example should instantiate");

        let ids = tree
            .ids_depth_first()
            .into_iter()
            .map(|id| id.0)
            .collect::<Vec<_>>();

        assert_eq!(ids.first().map(String::as_str), Some("redline-dashboard"));
        assert!(ids.iter().any(|id| id == "redline-waveform"));
        assert!(ids.iter().any(|id| id == "redline-queue-list"));
        assert!(ids.iter().any(|id| id == "redline-purge"));
    }

    #[test]
    fn registry_builds_tree_from_table_demo_example() {
        let input = std::fs::read_to_string(fixture_path("examples/table-demo.toml"))
            .expect("table demo example should exist");
        let spec = AppSpec::from_toml_str(&input).expect("table demo example should parse");

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("table demo example should instantiate");

        assert!(tree
            .ids_depth_first()
            .into_iter()
            .any(|id| id == ComponentId("service-table".into())));
        assert_eq!(
            tree.focusable_ids_depth_first(),
            vec![ComponentId("service-table".into())]
        );
    }

    #[test]
    fn registry_builds_tree_from_layout_pattern_examples() {
        for path in [
            "examples/layout-dense.toml",
            "examples/layout-sidebar.toml",
            "examples/layout-responsive.toml",
        ] {
            let input = std::fs::read_to_string(fixture_path(path))
                .expect("layout pattern example should exist");
            let spec = AppSpec::from_toml_str(&input).expect("layout pattern example should parse");

            let tree = ComponentRegistry::new()
                .build_tree(&spec)
                .expect("layout pattern example should instantiate");

            assert!(!tree.ids_depth_first().is_empty());
        }
    }

    #[test]
    fn registry_builds_tree_from_interactive_flow_example() {
        let input = std::fs::read_to_string(fixture_path("examples/interactive-flow.toml"))
            .expect("interactive flow example should exist");
        let spec = AppSpec::from_toml_str(&input).expect("interactive flow example should parse");

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("interactive flow example should instantiate");

        assert_eq!(
            tree.focusable_ids_depth_first(),
            vec![
                ComponentId("queue-list".into()),
                ComponentId("approve-action".into()),
                ComponentId("defer-action".into()),
            ]
        );
    }

    #[test]
    fn registry_builds_tree_from_cockpit_showcase_example() {
        let input = std::fs::read_to_string(fixture_path("examples/cockpit-showcase.toml"))
            .expect("cockpit showcase example should exist");
        let spec = AppSpec::from_toml_str(&input).expect("cockpit showcase example should parse");

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("cockpit showcase example should instantiate");

        assert!(!tree.ids_depth_first().is_empty());
    }

    #[test]
    fn registry_builds_tree_from_visual_system_showcase_example() {
        let input = std::fs::read_to_string(fixture_path("examples/visual-system-showcase.toml"))
            .expect("visual system showcase example should exist");
        let spec =
            AppSpec::from_toml_str(&input).expect("visual system showcase example should parse");

        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("visual system showcase example should instantiate");

        let ids = tree
            .ids_depth_first()
            .into_iter()
            .map(|id| id.0)
            .collect::<Vec<_>>();

        assert_eq!(ids.first().map(String::as_str), Some("visual-system"));
        assert!(ids.iter().any(|id| id == "hero-panel"));
        assert!(ids.iter().any(|id| id == "queue-panel"));
    }

    #[test]
    fn interactive_flow_routes_list_keys_scroll_and_button_activation() {
        let input = std::fs::read_to_string(fixture_path("examples/interactive-flow.toml"))
            .expect("interactive flow example should exist");
        let spec = AppSpec::from_toml_str(&input).expect("interactive flow example should parse");
        let mut tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("interactive flow example should instantiate");
        let mut ctx = EventContext::default();
        let list_id = ComponentId("queue-list".into());
        let button_id = ComponentId("approve-action".into());

        assert_eq!(
            tree.dispatch_event_to_target(&mut ctx, &list_id, &Event::FocusGained(list_id.clone())),
            EventResult::RequestRender
        );
        assert_eq!(
            tree.dispatch_event_to_target(
                &mut ctx,
                &list_id,
                &Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::default(),
                }),
            ),
            EventResult::RequestRender
        );
        assert_eq!(
            tree.dispatch_event_to_target(
                &mut ctx,
                &list_id,
                &Event::Scroll(ScrollEvent {
                    direction: ScrollDirection::Down,
                    amount: 1,
                }),
            ),
            EventResult::RequestRender
        );

        let _ =
            tree.dispatch_event_to_target(&mut ctx, &list_id, &Event::FocusLost(list_id.clone()));
        assert_eq!(
            tree.dispatch_event_to_target(
                &mut ctx,
                &button_id,
                &Event::FocusGained(button_id.clone()),
            ),
            EventResult::RequestRender
        );
        assert_eq!(
            tree.dispatch_event_to_target(
                &mut ctx,
                &button_id,
                &Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::default(),
                }),
            ),
            EventResult::RequestRender
        );
    }

    #[test]
    fn registry_builds_tree_from_application_templates() {
        for path in [
            "templates/operational-dashboard.toml",
            "templates/task-list.toml",
            "templates/metrics-monitor.toml",
        ] {
            let input =
                std::fs::read_to_string(fixture_path(path)).expect("template fixture should exist");
            let spec = AppSpec::from_toml_str(&input).expect("template should parse");

            let tree = ComponentRegistry::new()
                .build_tree(&spec)
                .expect("template should instantiate");

            assert!(tree.component_count() >= 8);
            assert!(tree.max_depth() >= 3);
        }
    }

    #[test]
    fn registry_builds_tree_with_knob_and_fui_panel() {
        let input = r#"
            schema_version = "0.1"
            theme = "redline"
            [root]
            kind = "Panel"
            id = "fui-panel"
            [root.props]
            border_style = "hex"
            grid = true
            controls = true

            [[root.children]]
            kind = "Knob"
            id = "warp-knob"
            [root.children.props]
            value = 75.0
            title = "WARP DIAL"
        "#;
        let spec = AppSpec::from_toml_str(input).expect("spec should parse");
        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("tree should build");

        assert_eq!(tree.component_count(), 2);
        assert_eq!(
            tree.ids_depth_first(),
            vec![
                ComponentId("fui-panel".into()),
                ComponentId("warp-knob".into()),
            ]
        );
    }

    #[test]
    fn registry_accepts_visual_panel_props() {
        let input = r#"
            schema_version = "0.1"
            theme = "redline"
            [root]
            kind = "Panel"
            id = "visual-panel"
            [root.props]
            title = "Visual"
            variant = "hero"
            density = "spacious"
            chrome = "cinematic"
        "#;
        let spec = AppSpec::from_toml_str(input).expect("spec should parse");
        let tree = ComponentRegistry::new()
            .build_tree(&spec)
            .expect("visual panel should build");

        assert_eq!(
            tree.ids_depth_first(),
            vec![ComponentId("visual-panel".into())]
        );
    }
}
