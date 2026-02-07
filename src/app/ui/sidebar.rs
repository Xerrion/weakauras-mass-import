//! Sidebar rendering: SavedVariables selection and existing aura tree.

use std::collections::HashSet;

use crate::saved_variables::AuraTreeNode;
use crate::theme;
use eframe::egui;

use super::super::WeakAuraImporter;

impl WeakAuraImporter {
    pub(crate) fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sv_panel")
            .min_width(250.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(theme::step_header(1, "Select SavedVariables"));
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // WoW path input
                ui.horizontal(|ui| {
                    ui.label("WoW Path:");
                    if ui.text_edit_singleline(&mut self.wow_path).changed() {
                        self.scan_saved_variables();
                    }
                    if ui.button("...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.wow_path = path.to_string_lossy().to_string();
                            self.scan_saved_variables();
                        }
                    }
                });

                ui.add_space(8.0);

                // Discovered files list
                ui.label(egui::RichText::new("Discovered files:").strong());

                egui::Frame::group(ui.style())
                    .fill(theme::colors::BG_ELEVATED)
                    .stroke(egui::Stroke::new(1.0, theme::colors::BORDER))
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                if self.discovered_sv_files.is_empty() {
                                    ui.label(theme::muted_text("No SavedVariables found"));
                                } else {
                                    for sv_info in &self.discovered_sv_files.clone() {
                                        let is_selected =
                                            self.selected_sv_path.as_ref() == Some(&sv_info.path);
                                        let text_color = if is_selected {
                                            theme::colors::GOLD
                                        } else {
                                            theme::colors::TEXT_SECONDARY
                                        };

                                        let label_text =
                                            format!("{} ({})", sv_info.account, sv_info.flavor);
                                        let label =
                                            egui::RichText::new(label_text).color(text_color);

                                        ui.horizontal(|ui| {
                                            if ui.selectable_label(is_selected, label).clicked() {
                                                self.selected_sv_path = Some(sv_info.path.clone());
                                                self.load_existing_auras();
                                            }

                                            if is_selected {
                                                ui.label(
                                                    egui::RichText::new("*")
                                                        .color(theme::colors::GOLD),
                                                );
                                            }
                                        });
                                    }
                                }
                            });
                    });

                ui.add_space(8.0);

                // Manual path selection
                if ui.button("Select file manually...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Lua files", &["lua"])
                        .pick_file()
                    {
                        self.selected_sv_path = Some(path);
                        self.load_existing_auras();
                    }
                }

                if let Some(path) = &self.selected_sv_path {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label("Selected:");
                    ui.label(
                        egui::RichText::new(path.file_name().unwrap_or_default().to_string_lossy())
                            .color(theme::colors::SUCCESS)
                            .strong(),
                    );
                }

                // Existing auras tree (or scanning indicator)
                if self.is_scanning {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            egui::RichText::new(&self.scanning_message)
                                .color(theme::colors::TEXT_SECONDARY),
                        );
                    });
                } else if !self.existing_auras_tree.is_empty() {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Existing Auras:").strong());
                        ui.label(theme::muted_text(&format!(
                            "({})",
                            self.existing_auras_count
                        )));
                    });

                    ui.add_space(4.0);

                    // Expand/Collapse all buttons
                    ui.horizontal(|ui| {
                        if ui.small_button("Expand all").clicked() {
                            fn collect_groups(node: &AuraTreeNode, set: &mut HashSet<String>) {
                                if node.is_group {
                                    set.insert(node.id.clone());
                                    for child in &node.children {
                                        collect_groups(child, set);
                                    }
                                }
                            }
                            for node in &self.existing_auras_tree {
                                collect_groups(node, &mut self.expanded_groups);
                            }
                        }
                        if ui.small_button("Collapse all").clicked() {
                            self.expanded_groups.clear();
                        }
                    });

                    // Selection & removal controls
                    ui.horizontal(|ui| {
                        if ui.small_button("Select all").clicked() {
                            fn collect_ids(node: &AuraTreeNode, set: &mut HashSet<String>) {
                                set.insert(node.id.clone());
                                for child in &node.children {
                                    collect_ids(child, set);
                                }
                            }
                            for node in &self.existing_auras_tree {
                                collect_ids(node, &mut self.selected_for_removal);
                            }
                        }
                        if ui.small_button("Deselect all").clicked() {
                            self.selected_for_removal.clear();
                        }

                        if !self.selected_for_removal.is_empty() {
                            let count = self.selected_for_removal.len();
                            let remove_btn = egui::Button::new(
                                egui::RichText::new(format!("Remove ({})", count))
                                    .color(theme::colors::BG_DARKEST)
                                    .small(),
                            )
                            .fill(theme::colors::ERROR);
                            if ui.add(remove_btn).clicked() {
                                // Only keep top-level selections (children of selected groups
                                // will be removed recursively by the backend)
                                self.pending_removal_ids =
                                    self.selected_for_removal.iter().cloned().collect();
                                self.show_remove_confirm = true;
                            }
                        }
                    });

                    ui.add_space(4.0);

                    // Scrollable aura tree
                    egui::Frame::group(ui.style())
                        .fill(theme::colors::BG_ELEVATED)
                        .stroke(egui::Stroke::new(1.0, theme::colors::BORDER))
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .id_salt("existing_auras_scroll")
                                .show(ui, |ui| {
                                    let tree = self.existing_auras_tree.clone();
                                    for node in &tree {
                                        self.render_aura_tree_node(ui, node, 0);
                                    }
                                });
                        });
                }

                // Import result
                if let Some(result) = &self.last_import_result {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.label(egui::RichText::new("Last import:").strong());
                    ui.label(result.summary());
                }
            });
    }

    pub(crate) fn render_aura_tree_node(
        &mut self,
        ui: &mut egui::Ui,
        node: &AuraTreeNode,
        depth: usize,
    ) {
        let indent = depth as f32 * 12.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);

            // Custom-painted checkbox for removal selection (always visible)
            let is_selected = self.selected_for_removal.contains(&node.id);
            let checkbox_size = egui::vec2(14.0, 14.0);
            let (rect, response) = ui.allocate_exact_size(checkbox_size, egui::Sense::click());

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                // Always-visible border
                painter.rect_stroke(
                    rect.shrink(1.0),
                    egui::Rounding::same(2.0),
                    egui::Stroke::new(1.5, theme::colors::TEXT_SECONDARY),
                );
                // Gold fill when checked
                if is_selected {
                    painter.rect_filled(
                        rect.shrink(3.0),
                        egui::Rounding::same(1.0),
                        theme::colors::GOLD,
                    );
                }
            }

            if response.clicked() {
                if is_selected {
                    self.selected_for_removal.remove(&node.id);
                } else {
                    self.selected_for_removal.insert(node.id.clone());
                }
            }

            if node.is_group {
                let is_expanded = self.expanded_groups.contains(&node.id);
                let icon = if is_expanded { "v" } else { ">" };

                if ui
                    .add(egui::Button::new(icon).small().frame(false))
                    .clicked()
                {
                    if is_expanded {
                        self.expanded_groups.remove(&node.id);
                    } else {
                        self.expanded_groups.insert(node.id.clone());
                    }
                }

                ui.label(
                    egui::RichText::new(&node.id)
                        .color(theme::colors::GOLD)
                        .strong(),
                );
                ui.label(theme::muted_text(&format!("({})", node.total_count() - 1)));
            } else {
                ui.add_space(18.0); // Align with group items
                ui.label(egui::RichText::new(&node.id).color(theme::colors::TEXT_SECONDARY));
            }
        });

        // Render children if expanded
        if node.is_group && self.expanded_groups.contains(&node.id) {
            for child in &node.children {
                self.render_aura_tree_node(ui, child, depth + 1);
            }
        }
    }
}
