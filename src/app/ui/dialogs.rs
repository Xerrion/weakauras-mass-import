//! Import confirmation and conflict resolution dialogs.

use crate::categories::UpdateCategory;
use crate::saved_variables::{ConflictAction, ImportConflict};
use crate::theme;
use eframe::egui;

use super::super::WeakAuraImporter;

impl WeakAuraImporter {
    pub(crate) fn render_import_confirmation(&mut self, ctx: &egui::Context) {
        if self.show_import_confirm {
            egui::Window::new("Confirm Import")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    let count = self
                        .parsed_auras
                        .iter()
                        .filter(|e| e.selected && e.validation.is_valid)
                        .count();
                    ui.label(format!("Import {} aura(s) to SavedVariables?", count));
                    ui.add_space(8.0);

                    if let Some(path) = &self.selected_sv_path {
                        ui.label(theme::muted_text(&format!(
                            "Target: {}",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        )));
                    }

                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_import_confirm = false;
                        }
                        let confirm_btn = egui::Button::new(
                            egui::RichText::new("Confirm Import").color(theme::colors::BG_DARKEST),
                        )
                        .fill(theme::colors::GOLD);
                        if ui.add(confirm_btn).clicked() {
                            self.show_import_confirm = false;
                            self.import_auras();
                        }
                    });
                });
        }
    }

    pub(crate) fn render_conflict_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_conflict_dialog {
            return;
        }

        // Clone data we need to avoid borrow issues
        let (new_count, conflict_count, conflicts) = match &self.conflict_result {
            Some(cr) => (cr.new_auras.len(), cr.conflicts.len(), cr.conflicts.clone()),
            None => return,
        };

        egui::Window::new("Import Conflicts Detected")
            .collapsible(false)
            .resizable(true)
            .min_width(600.0)
            .min_height(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // Header info
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{} new aura(s) will be added", new_count))
                            .color(theme::colors::SUCCESS),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("{} aura(s) already exist", conflict_count))
                            .color(theme::colors::GOLD),
                    );
                });
                ui.add_space(8.0);
                ui.separator();

                // Global category selection
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Default Categories to Update:").strong());
                ui.add_space(4.0);

                // Category checkboxes in a grid
                egui::Grid::new("global_categories")
                    .num_columns(4)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        for (i, category) in UpdateCategory::all().iter().enumerate() {
                            let mut enabled = self.global_categories.contains(category);
                            if ui.checkbox(&mut enabled, category.display_name()).changed() {
                                if enabled {
                                    self.global_categories.insert(*category);
                                } else {
                                    self.global_categories.remove(category);
                                }
                                // Update all resolutions that use UpdateSelected
                                for res in &mut self.conflict_resolutions {
                                    if res.action == ConflictAction::UpdateSelected {
                                        res.categories = self.global_categories.clone();
                                    }
                                }
                            }
                            if (i + 1) % 4 == 0 {
                                ui.end_row();
                            }
                        }
                    });

                ui.add_space(8.0);
                ui.separator();

                // Conflict list
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Conflicting Auras:").strong());

                // Bulk actions
                ui.horizontal(|ui| {
                    if ui.button("Import All").clicked() {
                        for res in &mut self.conflict_resolutions {
                            res.action = ConflictAction::UpdateSelected;
                            res.categories = self.global_categories.clone();
                        }
                    }
                    if ui.button("Skip All").clicked() {
                        for res in &mut self.conflict_resolutions {
                            res.action = ConflictAction::Skip;
                        }
                    }
                    if ui.button("Replace All").clicked() {
                        for res in &mut self.conflict_resolutions {
                            res.action = ConflictAction::ReplaceAll;
                        }
                    }
                });

                ui.add_space(4.0);

                // Scrollable conflict list
                let available_height = ui.available_height() - 50.0; // Leave room for buttons
                egui::Frame::group(ui.style())
                    .fill(theme::colors::BG_ELEVATED)
                    .stroke(egui::Stroke::new(1.0, theme::colors::BORDER))
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(available_height.max(150.0))
                            .show(ui, |ui| {
                                for (idx, conflict) in conflicts.iter().enumerate() {
                                    self.render_conflict_item(ui, idx, conflict);
                                }
                            });
                    });

                ui.add_space(8.0);

                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_conflict_dialog = false;
                        self.conflict_result = None;
                        self.conflict_resolutions.clear();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let import_btn = egui::Button::new(
                            egui::RichText::new("Import")
                                .color(theme::colors::BG_DARKEST)
                                .strong(),
                        )
                        .fill(theme::colors::GOLD)
                        .min_size(egui::vec2(100.0, 0.0));

                        if ui.add(import_btn).clicked() {
                            self.complete_import_with_resolutions();
                        }
                    });
                });
            });
    }

    pub(crate) fn render_remove_confirmation(&mut self, ctx: &egui::Context) {
        if !self.show_remove_confirm {
            return;
        }

        egui::Window::new("Confirm Removal")
            .collapsible(false)
            .resizable(true)
            .min_width(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                let count = self.pending_removal_ids.len();
                ui.label(
                    egui::RichText::new(format!("Remove {} aura(s) from SavedVariables?", count))
                        .strong(),
                );
                ui.add_space(4.0);
                ui.label(theme::muted_text(
                    "Groups will have all their children removed recursively.",
                ));
                ui.add_space(8.0);

                // Scrollable list of IDs to be removed
                egui::Frame::group(ui.style())
                    .fill(theme::colors::BG_ELEVATED)
                    .stroke(egui::Stroke::new(1.0, theme::colors::BORDER))
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for id in &self.pending_removal_ids.clone() {
                                    ui.label(
                                        egui::RichText::new(id)
                                            .color(theme::colors::TEXT_SECONDARY),
                                    );
                                }
                            });
                    });

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_remove_confirm = false;
                        self.pending_removal_ids.clear();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let remove_btn = egui::Button::new(
                            egui::RichText::new("Remove")
                                .color(theme::colors::BG_DARKEST)
                                .strong(),
                        )
                        .fill(theme::colors::ERROR);

                        if ui.add(remove_btn).clicked() {
                            self.show_remove_confirm = false;
                            self.remove_confirmed_auras();
                        }
                    });
                });
            });
    }

    fn render_conflict_item(&mut self, ui: &mut egui::Ui, idx: usize, conflict: &ImportConflict) {
        let resolution = &mut self.conflict_resolutions[idx];

        ui.horizontal(|ui| {
            // Action selector
            let action_text = match resolution.action {
                ConflictAction::Skip => "Skip",
                ConflictAction::ReplaceAll => "Replace",
                ConflictAction::UpdateSelected => "Update",
            };

            egui::ComboBox::from_id_salt(format!("action_{}", idx))
                .selected_text(action_text)
                .width(80.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut resolution.action, ConflictAction::Skip, "Skip");
                    ui.selectable_value(
                        &mut resolution.action,
                        ConflictAction::ReplaceAll,
                        "Replace",
                    );
                    ui.selectable_value(
                        &mut resolution.action,
                        ConflictAction::UpdateSelected,
                        "Update",
                    );
                });

            // Aura name
            let name_color = match resolution.action {
                ConflictAction::Skip => theme::colors::TEXT_MUTED,
                _ => theme::colors::TEXT_PRIMARY,
            };
            ui.label(egui::RichText::new(&conflict.aura_id).color(name_color));

            // Group indicator
            if conflict.is_group {
                ui.label(theme::muted_text(&format!(
                    "[Group: {} children]",
                    conflict.child_count
                )));
            }

            // Changed categories indicator
            if !conflict.changed_categories.is_empty() {
                let changed_names: Vec<&str> = conflict
                    .changed_categories
                    .iter()
                    .map(|c| c.display_name())
                    .collect();
                ui.label(theme::muted_text(&format!(
                    "Changes: {}",
                    changed_names.join(", ")
                )));
            }

            // Expand button for per-aura category selection
            if resolution.action == ConflictAction::UpdateSelected
                && ui
                    .button(if resolution.expanded { "v" } else { ">" })
                    .clicked()
            {
                resolution.expanded = !resolution.expanded;
            }
        });

        // Expanded category selection for this specific aura
        if resolution.expanded && resolution.action == ConflictAction::UpdateSelected {
            ui.indent(format!("categories_{}", idx), |ui| {
                egui::Grid::new(format!("aura_categories_{}", idx))
                    .num_columns(4)
                    .spacing([15.0, 2.0])
                    .show(ui, |ui| {
                        for (i, category) in UpdateCategory::all().iter().enumerate() {
                            let mut enabled = resolution.categories.contains(category);
                            let has_changes = conflict.changed_categories.contains(category);

                            let label = if has_changes {
                                egui::RichText::new(category.display_name())
                                    .color(theme::colors::GOLD)
                            } else {
                                egui::RichText::new(category.display_name())
                            };

                            if ui.checkbox(&mut enabled, label).changed() {
                                if enabled {
                                    resolution.categories.insert(*category);
                                } else {
                                    resolution.categories.remove(category);
                                }
                            }
                            if (i + 1) % 4 == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
        }

        ui.separator();
    }
}
