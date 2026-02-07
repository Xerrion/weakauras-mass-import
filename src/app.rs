//! Main GUI application for WeakAura Mass Import

use crate::categories::UpdateCategory;
use crate::decoder::{ValidationResult, WeakAura, WeakAuraDecoder};
use crate::error::WeakAuraError;
use crate::saved_variables::{
    AuraTreeNode, ConflictAction, ConflictDetectionResult, ConflictResolution, ImportConflict,
    ImportResult, SavedVariablesInfo, SavedVariablesManager,
};
use crate::theme;
use arboard::Clipboard;
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;

/// Main application state
pub struct WeakAuraImporter {
    /// Input text area content
    input_text: String,
    /// Parsed auras from input
    parsed_auras: Vec<ParsedAuraEntry>,
    /// Selected SavedVariables file
    selected_sv_path: Option<PathBuf>,
    /// Discovered SavedVariables files
    discovered_sv_files: Vec<SavedVariablesInfo>,
    /// Status message
    status_message: String,
    /// Status is error
    status_is_error: bool,
    /// Import result
    last_import_result: Option<ImportResult>,
    /// Show decoded view
    show_decoded_view: bool,
    /// Selected aura index for preview
    selected_aura_index: Option<usize>,
    /// Show import confirmation dialog
    show_import_confirm: bool,
    /// WoW path for scanning
    wow_path: String,
    /// Clipboard handler
    clipboard: Option<Clipboard>,
    /// Show paste input area
    show_paste_input: bool,
    /// Loaded existing auras as tree
    existing_auras_tree: Vec<AuraTreeNode>,
    /// Total count of existing auras
    existing_auras_count: usize,
    /// Show conflict resolution dialog
    show_conflict_dialog: bool,
    /// Current conflict detection result
    conflict_result: Option<ConflictDetectionResult>,
    /// Resolutions for each conflict (parallel to conflict_result.conflicts)
    conflict_resolutions: Vec<ConflictResolutionUI>,
    /// Global categories to update (used as default for all conflicts)
    global_categories: HashSet<UpdateCategory>,
    /// Selected conflict index in the dialog
    selected_conflict_index: Option<usize>,
    /// Expanded groups in the existing auras tree
    expanded_groups: HashSet<String>,
    /// Import progress (0.0 to 1.0)
    import_progress: f32,
    /// Is import currently in progress
    is_importing: bool,
    /// Import progress message
    import_progress_message: String,
}

/// Entry for a parsed aura in the list
struct ParsedAuraEntry {
    validation: ValidationResult,
    aura: Option<WeakAura>,
    selected: bool,
    /// Source file (if loaded from folder)
    source_file: Option<String>,
}

/// UI state for a single conflict resolution
#[derive(Clone)]
struct ConflictResolutionUI {
    /// The conflict this resolves
    aura_id: String,
    /// Current action
    action: ConflictAction,
    /// Categories to update (when action is UpdateSelected)
    categories: HashSet<UpdateCategory>,
    /// Whether to show details
    expanded: bool,
}

impl Default for WeakAuraImporter {
    fn default() -> Self {
        Self {
            input_text: String::new(),
            parsed_auras: Vec::new(),
            selected_sv_path: None,
            discovered_sv_files: Vec::new(),
            status_message: String::from("Ready. Load WeakAura strings from file or folder."),
            status_is_error: false,
            last_import_result: None,
            show_decoded_view: false,
            selected_aura_index: None,
            show_import_confirm: false,
            wow_path: String::new(),
            clipboard: Clipboard::new().ok(),
            show_paste_input: false,
            existing_auras_tree: Vec::new(),
            existing_auras_count: 0,
            show_conflict_dialog: false,
            conflict_result: None,
            conflict_resolutions: Vec::new(),
            global_categories: UpdateCategory::defaults(),
            selected_conflict_index: None,
            expanded_groups: HashSet::new(),
            import_progress: 0.0,
            is_importing: false,
            import_progress_message: String::new(),
        }
    }
}

impl WeakAuraImporter {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();

        // Auto-discover WoW installations
        let wow_paths = SavedVariablesManager::find_wow_paths();
        if let Some(first_path) = wow_paths.first() {
            app.wow_path = first_path.to_string_lossy().to_string();
            app.scan_saved_variables();
        }

        app
    }

    /// Scan for SavedVariables files
    fn scan_saved_variables(&mut self) {
        let path = PathBuf::from(&self.wow_path);
        if path.exists() {
            self.discovered_sv_files = SavedVariablesManager::find_saved_variables(&path);
            if !self.discovered_sv_files.is_empty() {
                self.set_status(&format!(
                    "Found {} SavedVariables file(s)",
                    self.discovered_sv_files.len()
                ));
            }
        }
    }

    /// Load existing auras from selected SavedVariables file
    fn load_existing_auras(&mut self) -> Option<SavedVariablesManager> {
        let sv_path = self.selected_sv_path.as_ref()?;
        let mut manager = SavedVariablesManager::new(sv_path.clone());

        match manager.load() {
            Ok(()) => {
                // Get aura tree structure
                let tree = manager.get_aura_tree();
                self.existing_auras_count = tree.iter().map(|n| n.total_count()).sum();
                self.existing_auras_tree = tree;
                self.expanded_groups.clear();
                Some(manager)
            }
            Err(WeakAuraError::FileNotFound(_)) => {
                self.existing_auras_tree = Vec::new();
                self.existing_auras_count = 0;
                self.expanded_groups.clear();
                Some(manager) // Return empty manager
            }
            Err(e) => {
                self.set_error(&format!("Failed to load SavedVariables: {}", e));
                self.existing_auras_tree = Vec::new();
                self.existing_auras_count = 0;
                None
            }
        }
    }

    /// Parse the input text for WeakAura strings
    fn parse_input(&mut self) {
        self.parsed_auras.clear();
        self.selected_aura_index = None;

        let results = WeakAuraDecoder::decode_multiple(&self.input_text);

        for result in results {
            let (validation, aura) = match result {
                Ok(aura) => {
                    let validation = ValidationResult {
                        is_valid: true,
                        format: Some(format!("v{}", aura.encoding_version)),
                        aura_id: Some(aura.id.clone()),
                        is_group: aura.is_group,
                        child_count: aura.children.len(),
                        error: None,
                    };
                    (validation, Some(aura))
                }
                Err(e) => {
                    let validation = ValidationResult {
                        is_valid: false,
                        format: None,
                        aura_id: None,
                        is_group: false,
                        child_count: 0,
                        error: Some(e.to_string()),
                    };
                    (validation, None)
                }
            };

            self.parsed_auras.push(ParsedAuraEntry {
                validation,
                aura,
                selected: true, // Select by default
                source_file: None,
            });
        }

        let valid_count = self
            .parsed_auras
            .iter()
            .filter(|e| e.validation.is_valid)
            .count();
        let total_count = self.parsed_auras.len();

        if total_count == 0 {
            self.set_status("No WeakAura strings detected in input");
        } else {
            self.set_status(&format!(
                "Parsed {} aura(s), {} valid",
                total_count, valid_count
            ));
        }
    }

    /// Import selected auras to SavedVariables
    fn import_auras(&mut self) {
        let Some(sv_path) = &self.selected_sv_path else {
            self.set_error("No SavedVariables file selected");
            return;
        };

        // Start import progress
        self.is_importing = true;
        self.import_progress = 0.0;
        self.import_progress_message = "Loading SavedVariables...".to_string();

        let mut manager = SavedVariablesManager::new(sv_path.clone());

        // Load existing SavedVariables
        self.import_progress = 0.1;
        if let Err(e) = manager.load() {
            // File might not exist yet, that's okay
            if !matches!(e, WeakAuraError::FileNotFound(_)) {
                self.set_error(&format!("Failed to load SavedVariables: {}", e));
                self.is_importing = false;
                return;
            }
        }

        self.import_progress = 0.2;
        self.import_progress_message = "Collecting auras...".to_string();

        // Collect selected valid auras
        let auras: Vec<&WeakAura> = self
            .parsed_auras
            .iter()
            .filter(|e| e.selected && e.aura.is_some())
            .filter_map(|e| e.aura.as_ref())
            .collect();

        if auras.is_empty() {
            self.set_error("No valid auras selected for import");
            self.is_importing = false;
            return;
        }

        self.import_progress = 0.3;
        self.import_progress_message = "Detecting conflicts...".to_string();

        // Detect conflicts
        let auras_owned: Vec<WeakAura> = auras.into_iter().cloned().collect();
        let conflict_result = manager.detect_conflicts(&auras_owned);

        // If there are conflicts, show the conflict dialog
        if !conflict_result.conflicts.is_empty() {
            // Initialize resolutions with defaults
            self.conflict_resolutions = conflict_result
                .conflicts
                .iter()
                .map(|c| ConflictResolutionUI {
                    aura_id: c.aura_id.clone(),
                    action: ConflictAction::UpdateSelected,
                    categories: self.global_categories.clone(),
                    expanded: false,
                })
                .collect();
            self.conflict_result = Some(conflict_result);
            self.show_conflict_dialog = true;
            self.selected_conflict_index = None;
            self.is_importing = false;
            self.import_progress = 0.0;
            self.import_progress_message.clear();
            return;
        }

        self.import_progress = 0.5;
        self.import_progress_message = "Importing auras...".to_string();

        // No conflicts - import directly
        let auras_owned: Vec<WeakAura> = self
            .parsed_auras
            .iter()
            .filter(|e| e.selected && e.aura.is_some())
            .filter_map(|e| e.aura.as_ref())
            .cloned()
            .collect();

        match manager.add_auras(&auras_owned) {
            Ok(result) => {
                self.import_progress = 0.8;
                self.import_progress_message = "Saving changes...".to_string();

                if let Err(e) = manager.save() {
                    self.set_error(&format!("Failed to save: {}", e));
                    self.is_importing = false;
                    return;
                }

                self.import_progress = 1.0;
                self.import_progress_message = "Complete!".to_string();

                self.set_status(&format!("Import complete: {}", result.summary()));
                self.last_import_result = Some(result);
                // Refresh existing auras tree
                let tree = manager.get_aura_tree();
                self.existing_auras_count = tree.iter().map(|n| n.total_count()).sum();
                self.existing_auras_tree = tree;

                // Reset progress after a short delay (handled by clearing state)
                self.is_importing = false;
            }
            Err(e) => {
                self.set_error(&format!("Import failed: {}", e));
                self.is_importing = false;
            }
        }
    }

    /// Complete import with conflict resolutions
    fn complete_import_with_resolutions(&mut self) {
        let Some(sv_path) = &self.selected_sv_path else {
            self.set_error("No SavedVariables file selected");
            return;
        };

        let Some(conflict_result) = self.conflict_result.take() else {
            return;
        };

        // Start import progress
        self.is_importing = true;
        self.import_progress = 0.0;
        self.import_progress_message = "Loading SavedVariables...".to_string();

        let mut manager = SavedVariablesManager::new(sv_path.clone());

        // Load existing
        self.import_progress = 0.2;
        if let Err(e) = manager.load() {
            if !matches!(e, WeakAuraError::FileNotFound(_)) {
                self.set_error(&format!("Failed to load SavedVariables: {}", e));
                self.is_importing = false;
                return;
            }
        }

        self.import_progress = 0.4;
        self.import_progress_message = "Applying resolutions...".to_string();

        // Convert UI resolutions to actual resolutions
        let resolutions: Vec<ConflictResolution> = self
            .conflict_resolutions
            .iter()
            .map(|r| ConflictResolution {
                aura_id: r.aura_id.clone(),
                action: r.action,
                categories_to_update: r.categories.clone(),
            })
            .collect();

        // Apply resolutions
        let result = manager.apply_resolutions(
            &conflict_result.new_auras,
            &conflict_result.conflicts,
            &resolutions,
        );

        self.import_progress = 0.7;
        self.import_progress_message = "Saving changes...".to_string();

        // Save
        if let Err(e) = manager.save() {
            self.set_error(&format!("Failed to save: {}", e));
            self.is_importing = false;
            return;
        }

        self.import_progress = 1.0;
        self.import_progress_message = "Complete!".to_string();

        self.set_status(&format!("Import complete: {}", result.summary()));
        self.last_import_result = Some(result);
        // Refresh existing auras tree
        let tree = manager.get_aura_tree();
        self.existing_auras_count = tree.iter().map(|n| n.total_count()).sum();
        self.existing_auras_tree = tree;
        self.show_conflict_dialog = false;
        self.conflict_resolutions.clear();
        self.is_importing = false;
    }

    /// Paste from clipboard
    fn paste_from_clipboard(&mut self) {
        if let Some(clipboard) = &mut self.clipboard {
            match clipboard.get_text() {
                Ok(text) => {
                    self.input_text = text;
                    self.parse_input();
                }
                Err(e) => {
                    self.set_error(&format!("Clipboard error: {}", e));
                }
            }
        }
    }

    /// Load from file
    fn load_from_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt", "md"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.input_text = content;
                    self.parse_input();
                    self.set_status(&format!("Loaded from {}", path.display()));
                }
                Err(e) => {
                    self.set_error(&format!("Failed to read file: {}", e));
                }
            }
        }
    }

    /// Load WeakAura strings from all files in a folder
    fn load_from_folder(&mut self) {
        if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
            self.parsed_auras.clear();
            self.selected_aura_index = None;
            self.input_text.clear();

            let mut total_files = 0;
            let mut total_auras = 0;
            let mut valid_auras = 0;

            // Recursively scan folder for supported files
            if let Ok(entries) = Self::scan_folder_recursive(&folder_path) {
                for file_path in entries {
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                        let filename = file_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string());

                        let results = WeakAuraDecoder::decode_multiple(&content);
                        if !results.is_empty() {
                            total_files += 1;
                        }

                        for result in results {
                            total_auras += 1;
                            let (validation, aura) = match result {
                                Ok(aura) => {
                                    valid_auras += 1;
                                    let validation = ValidationResult {
                                        is_valid: true,
                                        format: Some(format!("v{}", aura.encoding_version)),
                                        aura_id: Some(aura.id.clone()),
                                        is_group: aura.is_group,
                                        child_count: aura.children.len(),
                                        error: None,
                                    };
                                    (validation, Some(aura))
                                }
                                Err(e) => {
                                    let validation = ValidationResult {
                                        is_valid: false,
                                        format: None,
                                        aura_id: None,
                                        is_group: false,
                                        child_count: 0,
                                        error: Some(e.to_string()),
                                    };
                                    (validation, None)
                                }
                            };

                            self.parsed_auras.push(ParsedAuraEntry {
                                validation,
                                aura,
                                selected: true,
                                source_file: Some(filename.clone()),
                            });
                        }
                    }
                }
            }

            if total_auras == 0 {
                self.set_status("No WeakAura strings found in folder");
            } else {
                self.set_status(&format!(
                    "Loaded {} aura(s) from {} file(s), {} valid",
                    total_auras, total_files, valid_auras
                ));
            }
        }
    }

    /// Recursively scan a folder for supported files (.txt, .md, .lua)
    fn scan_folder_recursive(folder: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let supported_extensions = ["txt", "md", "lua"];

        fn visit_dir(
            dir: &PathBuf,
            files: &mut Vec<PathBuf>,
            extensions: &[&str],
        ) -> std::io::Result<()> {
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dir(&path, files, extensions)?;
                    } else if let Some(ext) = path.extension() {
                        if extensions.iter().any(|e| ext.eq_ignore_ascii_case(e)) {
                            files.push(path);
                        }
                    }
                }
            }
            Ok(())
        }

        visit_dir(folder, &mut files, &supported_extensions)?;
        Ok(files)
    }

    fn set_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
        self.status_is_error = false;
    }

    fn set_error(&mut self, msg: &str) {
        self.status_message = msg.to_string();
        self.status_is_error = true;
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
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

    fn render_status_bar(&mut self, ctx: &egui::Context) {
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

    fn render_sidebar(&mut self, ctx: &egui::Context) {
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

                // Existing auras tree
                if !self.existing_auras_tree.is_empty() {
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
                            fn collect_groups(
                                node: &AuraTreeNode,
                                set: &mut std::collections::HashSet<String>,
                            ) {
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

    fn render_aura_tree_node(&mut self, ui: &mut egui::Ui, node: &AuraTreeNode, depth: usize) {
        let indent = depth as f32 * 12.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);

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

    fn render_decoded_panel(&mut self, ctx: &egui::Context) {
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

    fn render_import_confirmation(&mut self, ctx: &egui::Context) {
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

    fn render_conflict_dialog(&mut self, ctx: &egui::Context) {
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

    fn render_main_content(&mut self, ctx: &egui::Context) {
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
                    self.load_from_file();
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

impl eframe::App for WeakAuraImporter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        theme::configure_theme(ctx);
        self.render_menu_bar(ctx);
        self.render_status_bar(ctx);
        self.render_sidebar(ctx);
        self.render_decoded_panel(ctx);
        self.render_main_content(ctx);
        self.render_import_confirmation(ctx);
        self.render_conflict_dialog(ctx);
    }
}
