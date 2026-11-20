use cpm_core::{ParameterDefinition, VariableType};
use eframe::egui::{self, TextBuffer};

pub fn input(
    ui: &mut egui::Ui,
    value: &mut String,
    definition: Option<&ParameterDefinition>,
    color: Option<egui::Color32>,
) -> egui::Response {
    if let Some(definition) = definition {
        match definition.variable_type {
            VariableType::Boolean
                if value.is_empty()
                    || value.eq_ignore_ascii_case("true")
                    || value.eq_ignore_ascii_case("false") =>
            {
                let mut checked = value.eq_ignore_ascii_case("true");
                let response = ui.checkbox(&mut checked, "");
                if response.changed() {
                    *value = checked.to_string();
                }
                return response;
            }
            VariableType::Enumeration if !definition.choices.is_empty() => {
                let old = value.clone();
                let mut response = egui::ComboBox::from_id_salt("enum-value")
                    .selected_text(value.as_str())
                    .show_ui(ui, |ui| {
                        for choice in &definition.choices {
                            ui.selectable_value(value, choice.clone(), choice);
                        }
                    })
                    .response;
                if *value != old {
                    response.mark_changed();
                }
                return response;
            }
            VariableType::Vector2 | VariableType::Vector3 | VariableType::Quaternion => {
                let labels: &[&str] = match definition.variable_type {
                    VariableType::Vector2 => &["X", "Y"],
                    VariableType::Vector3 => &["X", "Y", "Z"],
                    _ => &["W", "X", "Y", "Z"],
                };
                let mut parts: Vec<String> = if value.is_empty() {
                    vec![String::new(); labels.len()]
                } else {
                    value.split(' ').map(str::to_owned).collect()
                };
                if parts.len() == labels.len() {
                    let mut changed = false;
                    let component_width = ((ui.available_width() - labels.len() as f32 * 30.0)
                        / labels.len() as f32)
                        .clamp(35.0, 85.0);
                    let decimal = ParameterDefinition {
                        variable_type: VariableType::Decimal,
                        ..Default::default()
                    };
                    let mut response = ui
                        .horizontal(|ui| {
                            for (i, (label, part)) in labels.iter().zip(&mut parts).enumerate() {
                                ui.label(*label);
                                ui.push_id(i, |ui| {
                                    ui.scope(|ui| {
                                        ui.set_max_width(component_width);
                                        changed |=
                                            text_input(ui, part, Some(&decimal), color).changed();
                                    });
                                });
                            }
                        })
                        .response;
                    if changed {
                        *value = parts.join(" ");
                        response.mark_changed();
                    }
                    return response;
                }
            }
            _ => {}
        }
    }
    text_input(ui, value, definition, color)
}
fn text_input(
    ui: &mut egui::Ui,
    value: &mut String,
    definition: Option<&ParameterDefinition>,
    color: Option<egui::Color32>,
) -> egui::Response {
    let old = value.clone();
    let mut buffer = FilteredBuffer {
        value,
        definition,
        rejected: false,
    };
    let hint = definition
        .map(input_hint)
        .unwrap_or_else(|| "Valeur du paramètre".into());
    let mut edit = egui::TextEdit::singleline(&mut buffer)
        .hint_text(hint)
        .desired_width(f32::INFINITY);
    if let Some(color) = color {
        edit = edit.text_color(color);
    }
    let response = ui.add(edit);
    if definition.is_some_and(|d| d.name == "MaxRPM" || d.name == "RPMRange") {
        ui.small("Unité : tr/min");
    }
    if buffer.rejected {
        *buffer.value = old;
        ui.ctx().request_repaint();
    }
    response
}
fn input_hint(definition: &ParameterDefinition) -> String {
    if let Some(value) = &definition.default_value {
        return format!("Défaut : {value}");
    }
    match definition.variable_type {
        VariableType::Integer => "Nombre entier, ex. 100".into(),
        VariableType::Decimal => "Nombre décimal, ex. 0.5".into(),
        VariableType::Vector2 => "Deux nombres : X Y".into(),
        VariableType::Vector3 => "Trois nombres : X Y Z".into(),
        VariableType::Quaternion => "Quatre nombres : W X Y Z".into(),
        _ => "Valeur du paramètre".into(),
    }
}
struct FilteredBuffer<'a> {
    value: &'a mut String,
    definition: Option<&'a ParameterDefinition>,
    rejected: bool,
}
impl TextBuffer for FilteredBuffer<'_> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<FilteredBuffer<'static>>()
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.value
    }
    fn insert_text(&mut self, text: &str, index: egui::text::CharIndex) -> usize {
        let text = if self.definition.is_some_and(|d| is_numeric(d.variable_type)) {
            text.replace(',', ".")
        } else {
            text.to_owned()
        };
        let mut next = self.value.clone();
        let inserted = TextBuffer::insert_text(&mut next, &text, index);
        if self.definition.is_none_or(|d| allowed(&next, d)) {
            *self.value = next;
            inserted
        } else {
            self.rejected = true;
            0
        }
    }
    fn delete_char_range(&mut self, range: std::ops::Range<egui::text::CharIndex>) {
        TextBuffer::delete_char_range(self.value, range);
    }
    fn replace_with(&mut self, text: &str) {
        if self.definition.is_none_or(|d| allowed(text, d)) {
            *self.value = text.to_owned();
        } else {
            self.rejected = true;
        }
    }
}
fn is_numeric(kind: VariableType) -> bool {
    matches!(
        kind,
        VariableType::Integer
            | VariableType::Decimal
            | VariableType::Vector2
            | VariableType::Vector3
            | VariableType::Quaternion
    )
}
fn number_partial(value: &str, integer: bool) -> bool {
    let value = value.strip_prefix(['+', '-']).unwrap_or(value);
    let mut digits = false;
    let mut dot = false;
    let mut exponent = false;
    let mut exponent_start = false;
    for c in value.chars() {
        if c.is_ascii_digit() {
            digits = true;
            exponent_start = false;
            continue;
        }
        if !integer && c == '.' && !dot && !exponent {
            dot = true;
            continue;
        }
        if !integer && matches!(c, 'e' | 'E') && !exponent && digits {
            exponent = true;
            exponent_start = true;
            continue;
        }
        if !integer && matches!(c, '+' | '-') && exponent_start {
            exponent_start = false;
            continue;
        }
        return false;
    }
    true
}
fn allowed(value: &str, definition: &ParameterDefinition) -> bool {
    let kind = definition.variable_type;
    if !is_numeric(kind) {
        return true;
    }
    if definition.name == "MaxVehicleSpeed"
        && !value.is_empty()
        && "infinite".starts_with(&value.to_ascii_lowercase())
    {
        return true;
    }
    match kind {
        VariableType::Integer => number_partial(value, true),
        VariableType::Decimal => number_partial(value, false),
        _ => {
            let max = match kind {
                VariableType::Vector2 => 2,
                VariableType::Vector3 => 3,
                _ => 4,
            };
            let mut count = 0;
            value.split_whitespace().all(|v| {
                count += 1;
                count <= max && number_partial(v, false)
            })
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numeric_buffer_blocks_typing_and_pasting_letters_but_preserves_editing_states() {
        let definition = ParameterDefinition {
            variable_type: VariableType::Decimal,
            ..Default::default()
        };
        for value in ["", "-", "-.", "-1.25", "1e", "1e-", "1e-3"] {
            assert!(allowed(value, &definition), "{value}");
        }
        for value in ["abc", "1a", "1.2.3", "1 2", "e2"] {
            assert!(!allowed(value, &definition), "{value}");
        }
        let mut value = "12".to_owned();
        let mut buffer = FilteredBuffer {
            value: &mut value,
            definition: Some(&definition),
            rejected: false,
        };
        assert_eq!(buffer.insert_text("abc", egui::text::CharIndex(2)), 0);
        assert_eq!(buffer.as_str(), "12");
        buffer.insert_text(",5", egui::text::CharIndex(2));
        assert_eq!(buffer.as_str(), "12.5");
        let integer = ParameterDefinition {
            variable_type: VariableType::Integer,
            ..Default::default()
        };
        assert!(!allowed("1.5", &integer));
        let vector = ParameterDefinition {
            variable_type: VariableType::Vector3,
            ..Default::default()
        };
        assert!(allowed("1 -2.5 3", &vector));
        assert!(!allowed("1 2 three", &vector));
        assert!(!allowed("1 2 3 4", &vector));
    }
}

#[cfg(test)]
mod widget_tests {
    use super::*;
    #[test]
    fn typed_widgets_preserve_imported_values_until_user_changes_them() {
        let ctx = egui::Context::default();
        for (kind, text) in [
            (VariableType::Vector3, "1 2 3"),
            (VariableType::Quaternion, "1 0 0 0"),
            (VariableType::Vector3, "1  2   3"),
            (VariableType::Boolean, "INVALID"),
            (VariableType::Boolean, ""),
            (VariableType::Enumeration, "unknown"),
        ] {
            let mut value = text.to_owned();
            let definition = ParameterDefinition {
                variable_type: kind,
                choices: vec!["first".into(), "second".into()],
                ..Default::default()
            };
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                input(ui, &mut value, Some(&definition), None);
            });
            output.textures_delta.clear();
            assert_eq!(value, text);
        }
    }
}
