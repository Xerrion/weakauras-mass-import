//! Top-level UI rendering: menu bar, status bar, decoded panel.

mod dialogs;
mod main_panel;
mod sidebar;

use eframe::egui;

use super::WeakAuraImporter;

impl WeakAuraImporter {
    pub(crate) fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load from file...").clicked() {
                        self.load_from_file();
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Paste from clipboard").clicked() {
                        self.paste_from_clipboard();
                        ui.close_menu();
                    }
                    if ui.button("Clear").clicked() {
                        self.input_text.clear();
                        self.parsed_auras.clear();
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_decoded_view, "Show decoded JSON");
                });
            });
        });
    }

    pub(crate) fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let color = if self.status_is_error {
                    egui::Color32::RED
                } else {
                    ui.visuals().text_color()
                };
                ui.colored_label(color, &self.status_message);
            });
        });
    }

    pub(crate) fn render_decoded_panel(&mut self, ctx: &egui::Context) {
        if self.show_decoded_view {
            egui::SidePanel::right("decoded_panel")
                .min_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Decoded Data");
                    ui.separator();

                    if let Some(idx) = self.selected_aura_index {
                        if let Some(entry) = self.parsed_auras.get(idx) {
                            if let Some(aura) = &entry.aura {
                                egui::ScrollArea::both().show(ui, |ui| {
                                    let json = serde_json::to_string_pretty(&aura.data)
                                        .unwrap_or_else(|_| "Failed to serialize".to_string());
                                    ui.add(
                                        egui::TextEdit::multiline(&mut json.as_str())
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(f32::INFINITY),
                                    );
                                });
                            }
                        }
                    } else {
                        ui.label("Select an aura to view decoded data");
                    }
                });
        }
    }
}
