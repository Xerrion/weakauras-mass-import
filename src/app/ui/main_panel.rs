//! Main content panel: input area, aura list, import controls.

use crate::theme;
use eframe::egui;

use super::super::WeakAuraImporter;

impl WeakAuraImporter {
    pub(crate) fn render_main_content(&mut self, ctx: &egui::Context) {
        // Keyboard shortcuts
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::V)) {
            self.show_paste_input = true;
            self.paste_from_clipboard();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter))
            && self.show_paste_input
        {
            self.parse_input();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Step 2 Header
            ui.add_space(4.0);
            ui.label(theme::step_header(2, "Load WeakAuras"));
            ui.add_space(8.0);

            // Action Buttons
            ui.horizontal(|ui| {
                // Paste toggle button
                let paste_btn = if self.show_paste_input {
                    egui::Button::new(egui::RichText::new("Paste").color(theme::colors::BG_DARKEST))
                        .fill(theme::colors::GOLD)
                } else {
                    egui::Button::new("Paste")
                };
                if ui
                    .add(paste_btn)
                    .on_hover_text("Toggle paste input area (Ctrl+V)")
                    .clicked()
                {
                    self.show_paste_input = !self.show_paste_input;
                }

                if ui
                    .button("Load file")
                    .on_hover_text("Load WeakAura strings from a text file")
                    .clicked()
                {
                    self.load_from_file_async();
                }
                if ui
                    .button("Load folder")
                    .on_hover_text("Scan folder recursively for WeakAura strings (.txt, .md, .lua)")
                    .clicked()
                {
                    self.load_from_folder();
                }
                if ui
                    .button("Clear")
                    .on_hover_text("Clear all input and parsed auras")
                    .clicked()
                {
                    self.input_text.clear();
                    self.parsed_auras.clear();
                    self.show_paste_input = false;
                }
            });

            // Paste input area (only shown when toggled)
            if self.show_paste_input {
                ui.add_space(8.0);

                // Calculate height for input area
                let input_height = if self.parsed_auras.is_empty() {
                    (ui.available_height() - 80.0).max(100.0)
                } else {
                    150.0
                };

                egui::Frame::group(ui.style())
                    .fill(theme::colors::BG_ELEVATED)
                    .stroke(egui::Stroke::new(1.0, theme::colors::BORDER))
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(input_height)
                            .id_salt("input_scroll")
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.input_text)
                                        .hint_text(
                                            "Paste WeakAura import strings here (one per line)",
                                        )
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(8),
                                );
                            });
                    });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    if ui
                        .button("Paste from clipboard")
                        .on_hover_text("Paste from clipboard (Ctrl+V)")
                        .clicked()
                    {
                        self.paste_from_clipboard();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let parse_btn = egui::Button::new(
                            egui::RichText::new("Parse")
                                .strong()
                                .color(theme::colors::BG_DARKEST),
                        )
                        .fill(theme::colors::GOLD)
                        .rounding(4.0)
                        .min_size(egui::vec2(80.0, 0.0));

                        if ui
                            .add(parse_btn)
                            .on_hover_text("Parse input text for WeakAura strings (Ctrl+Enter)")
                            .clicked()
                        {
                            self.parse_input();
                        }
                    });
                });
            }

            // Step 3: Review & Import (only if auras parsed)
            if !self.parsed_auras.is_empty() {
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);

                ui.label(theme::step_header(3, "Review & Import"));
                ui.add_space(4.0);

                // Selection Controls, Import Button & Stats
                let can_import = self.selected_sv_path.is_some()
                    && self
                        .parsed_auras
                        .iter()
                        .any(|e| e.selected && e.validation.is_valid)
                    && !self.is_importing;

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!self.is_importing, egui::Button::new("Select All"))
                        .on_hover_text("Select all valid auras for import")
                        .clicked()
                    {
                        for entry in &mut self.parsed_auras {
                            if entry.validation.is_valid {
                                entry.selected = true;
                            }
                        }
                    }
                    if ui
                        .add_enabled(!self.is_importing, egui::Button::new("Deselect All"))
                        .on_hover_text("Deselect all auras")
                        .clicked()
                    {
                        for entry in &mut self.parsed_auras {
                            entry.selected = false;
                        }
                    }

                    ui.add_space(16.0);

                    // Import button inline
                    ui.add_enabled_ui(can_import, |ui| {
                        let button_text = egui::RichText::new("Import Selected >>")
                            .strong()
                            .size(14.0);
                        let button = if can_import {
                            ui.add(
                                egui::Button::new(button_text.color(theme::colors::BG_DARKEST))
                                    .fill(theme::colors::GOLD)
                                    .min_size(egui::vec2(140.0, 24.0)),
                            )
                        } else {
                            ui.add(
                                egui::Button::new(button_text.color(theme::colors::TEXT_MUTED))
                                    .min_size(egui::vec2(140.0, 24.0)),
                            )
                        };

                        if button
                            .on_hover_text("Import selected auras to SavedVariables")
                            .clicked()
                        {
                            self.show_import_confirm = true;
                        }
                    });

                    if !can_import && self.selected_sv_path.is_none() && !self.is_importing {
                        ui.label(theme::muted_text("Select a SavedVariables file first"));
                    }

                    // Stats aligned right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let selected_count =
                            self.parsed_auras.iter().filter(|e| e.selected).count();
                        let valid_count = self
                            .parsed_auras
                            .iter()
                            .filter(|e| e.validation.is_valid)
                            .count();

                        ui.label(
                            egui::RichText::new(format!(
                                "{} selected / {} valid / {} total",
                                selected_count,
                                valid_count,
                                self.parsed_auras.len()
                            ))
                            .color(theme::colors::TEXT_SECONDARY),
                        );
                    });
                });

                // Progress bar (shown during import)
                if self.is_importing {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::ProgressBar::new(self.import_progress)
                                .show_percentage()
                                .animate(true),
                        );
                    });
                    if !self.import_progress_message.is_empty() {
                        ui.label(
                            egui::RichText::new(&self.import_progress_message)
                                .color(theme::colors::TEXT_SECONDARY)
                                .small(),
                        );
                    }
                }

                ui.add_space(8.0);

                // Aura List (scrollable) - fills remaining space
                let available_height = ui.available_height();
                let available_width = ui.available_width();
                egui::Frame::group(ui.style())
                    .fill(theme::colors::BG_ELEVATED)
                    .stroke(egui::Stroke::new(1.0, theme::colors::BORDER))
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        ui.set_min_width(available_width - 16.0);
                        egui::ScrollArea::both()
                            .id_salt("auras_scroll")
                            .max_height(available_height - 16.0)
                            .show(ui, |ui| {
                                for (idx, entry) in self.parsed_auras.iter_mut().enumerate() {
                                    let is_selected_for_view =
                                        self.selected_aura_index == Some(idx);
                                    let is_valid = entry.validation.is_valid;

                                    ui.horizontal(|ui| {
                                        // Checkbox (only if valid)
                                        ui.add_enabled(
                                            is_valid,
                                            egui::Checkbox::new(&mut entry.selected, ""),
                                        );

                                        // Status Icon
                                        let icon = if is_valid {
                                            egui::RichText::new("*").color(theme::colors::SUCCESS)
                                        } else {
                                            egui::RichText::new("x").color(theme::colors::ERROR)
                                        };
                                        ui.label(icon);

                                        // Aura Info Label (clickable)
                                        let summary = entry.validation.summary();
                                        let label_text = if is_valid {
                                            egui::RichText::new(&summary)
                                                .color(theme::colors::TEXT_PRIMARY)
                                        } else {
                                            egui::RichText::new(&summary)
                                                .color(theme::colors::TEXT_MUTED)
                                        };

                                        let response =
                                            ui.selectable_label(is_selected_for_view, label_text);
                                        if response.clicked() {
                                            self.selected_aura_index = Some(idx);
                                        }

                                        // Group Child Count
                                        if entry.validation.is_group {
                                            ui.label(theme::muted_text(&format!(
                                                "[Group: {} children]",
                                                entry.validation.child_count
                                            )));
                                        }

                                        // Source file (if loaded from folder)
                                        if let Some(ref source) = entry.source_file {
                                            ui.label(theme::muted_text(&format!("< {}", source)));
                                        }
                                    });
                                }
                            });
                    });
            }
        });
    }
}
