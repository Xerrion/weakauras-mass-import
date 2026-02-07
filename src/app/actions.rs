//! Business logic methods for WeakAuraImporter.

use std::path::PathBuf;

use crate::decoder::{ValidationResult, WeakAura, WeakAuraDecoder};
use crate::error::WeakAuraError;
use crate::saved_variables::{ConflictAction, ConflictResolution, SavedVariablesManager};

use super::state::{ConflictResolutionUI, ParsedAuraEntry};
use super::WeakAuraImporter;

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
    pub(crate) fn scan_saved_variables(&mut self) {
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
    pub(crate) fn load_existing_auras(&mut self) -> Option<SavedVariablesManager> {
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
    pub(crate) fn parse_input(&mut self) {
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
    pub(crate) fn import_auras(&mut self) {
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
    pub(crate) fn complete_import_with_resolutions(&mut self) {
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
    pub(crate) fn paste_from_clipboard(&mut self) {
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
    pub(crate) fn load_from_file(&mut self) {
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
    pub(crate) fn load_from_folder(&mut self) {
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

    pub(crate) fn set_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
        self.status_is_error = false;
    }

    pub(crate) fn set_error(&mut self, msg: &str) {
        self.status_message = msg.to_string();
        self.status_is_error = true;
    }

    /// Remove selected auras from SavedVariables (called after confirmation).
    pub(crate) fn remove_confirmed_auras(&mut self) {
        let Some(sv_path) = &self.selected_sv_path else {
            self.set_error("No SavedVariables file selected");
            return;
        };

        let ids = std::mem::take(&mut self.pending_removal_ids);
        if ids.is_empty() {
            return;
        }

        let mut manager = SavedVariablesManager::new(sv_path.clone());

        if let Err(e) = manager.load() {
            if !matches!(e, WeakAuraError::FileNotFound(_)) {
                self.set_error(&format!("Failed to load SavedVariables: {}", e));
                return;
            }
        }

        let removed = manager.remove_auras(&ids);

        if removed.is_empty() {
            self.set_status("No auras were removed (already absent)");
            return;
        }

        if let Err(e) = manager.save() {
            self.set_error(&format!("Failed to save: {}", e));
            return;
        }

        self.set_status(&format!("Removed {} aura(s)", removed.len()));

        // Refresh existing auras tree
        let tree = manager.get_aura_tree();
        self.existing_auras_count = tree.iter().map(|n| n.total_count()).sum();
        self.existing_auras_tree = tree;
        self.selected_for_removal.clear();
    }
}
