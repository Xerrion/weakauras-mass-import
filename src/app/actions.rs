//! Business logic methods for WeakAuraImporter (iced version).

use std::collections::HashSet;
use std::path::PathBuf;

use iced::Task;

use crate::decoder::{ValidationResult, WeakAura, WeakAuraDecoder};
use crate::error::WeakAuraError;
use crate::saved_variables::{ConflictAction, ConflictResolution, SavedVariablesManager};

use super::state::{ConflictResolutionUI, ImportUpdate, LoadingUpdate, ParsedAuraEntry, RemovalUpdate, ScanUpdate};
use super::{Message, WeakAuraImporter};

/// Collect the set of aura IDs already present in the parsed auras list.
fn collect_existing_ids(parsed_auras: &[ParsedAuraEntry]) -> HashSet<String> {
    parsed_auras
        .iter()
        .filter_map(|e| e.validation.aura_id.as_deref())
        .map(|id| id.to_string())
        .collect()
}

/// Decode auras from content, filtering out duplicates already in `existing_ids`.
/// Returns `(entries, added, duplicates, invalid)`.
fn decode_auras_filtered(
    content: &str,
    source_file: Option<String>,
    existing_ids: &HashSet<String>,
) -> (Vec<ParsedAuraEntry>, usize, usize, usize) {
    let results = WeakAuraDecoder::decode_multiple(content);
    let mut entries = Vec::new();
    let mut added = 0;
    let mut duplicates = 0;
    let mut invalid = 0;

    for result in results {
        match result {
            Ok(aura) => {
                if existing_ids.contains(&aura.id) {
                    duplicates += 1;
                    continue;
                }
                added += 1;
                let validation = ValidationResult {
                    is_valid: true,
                    format: Some(format!("v{}", aura.encoding_version)),
                    aura_id: Some(aura.id.clone()),
                    is_group: aura.is_group,
                    child_count: aura.children.len(),
                    error: None,
                };
                entries.push(ParsedAuraEntry {
                    validation,
                    aura: Some(aura),
                    selected: true,
                    source_file: source_file.clone(),
                });
            }
            Err(e) => {
                invalid += 1;
                let validation = ValidationResult {
                    is_valid: false,
                    format: None,
                    aura_id: None,
                    is_group: false,
                    child_count: 0,
                    error: Some(e.to_string()),
                };
                entries.push(ParsedAuraEntry {
                    validation,
                    aura: None,
                    selected: true,
                    source_file: source_file.clone(),
                });
            }
        }
    }

    (entries, added, duplicates, invalid)
}

impl WeakAuraImporter {
    /// Scan for SavedVariables files (synchronous, called during init)
    pub(crate) fn scan_saved_variables_sync(&mut self) {
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

    /// Load existing auras from selected SavedVariables file (async)
    pub(crate) fn load_existing_auras_async(&mut self) -> Task<Message> {
        let Some(sv_path) = self.selected_sv_path.clone() else {
            return Task::none();
        };

        self.is_scanning = true;
        self.scanning_message = "Loading SavedVariables...".to_string();

        Task::perform(
            async move {
                let mut manager = SavedVariablesManager::new(sv_path);

                match manager.load() {
                    Ok(()) => {
                        let tree = manager.get_aura_tree();
                        let count = tree.iter().map(|n| n.total_count()).sum();
                        ScanUpdate::Complete { tree, count }
                    }
                    Err(WeakAuraError::FileNotFound(_)) => {
                        // File doesn't exist yet — that's okay, just return empty
                        ScanUpdate::Complete {
                            tree: Vec::new(),
                            count: 0,
                        }
                    }
                    Err(e) => ScanUpdate::Error(format!("Failed to load SavedVariables: {}", e)),
                }
            },
            Message::ScanUpdate,
        )
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
                selected: true,
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

    /// Import selected auras to SavedVariables (async)
    pub(crate) fn import_auras_async(&mut self) -> Task<Message> {
        let Some(sv_path) = self.selected_sv_path.clone() else {
            self.set_error("No SavedVariables file selected");
            return Task::none();
        };

        // Collect selected valid auras
        let auras: Vec<WeakAura> = self
            .parsed_auras
            .iter()
            .filter(|e| e.selected && e.aura.is_some())
            .filter_map(|e| e.aura.clone())
            .collect();

        if auras.is_empty() {
            self.set_error("No valid auras selected for import");
            return Task::none();
        }

        self.is_importing = true;
        self.import_progress = 0.0;
        self.import_progress_message = "Starting import...".to_string();

        Task::perform(
            async move {
                let mut manager = SavedVariablesManager::new(sv_path);

                // Load existing SavedVariables
                if let Err(e) = manager.load() {
                    if !matches!(e, WeakAuraError::FileNotFound(_)) {
                        return ImportUpdate::Error(format!("Failed to load SavedVariables: {}", e));
                    }
                }

                // Detect conflicts
                let conflict_result = manager.detect_conflicts(&auras);

                // If there are conflicts, send back to UI for resolution
                if !conflict_result.conflicts.is_empty() {
                    return ImportUpdate::ConflictsDetected(conflict_result);
                }

                // No conflicts - import directly
                match manager.add_auras(&auras) {
                    Ok(result) => {
                        if let Err(e) = manager.save() {
                            return ImportUpdate::Error(format!("Failed to save: {}", e));
                        }

                        let tree = manager.get_aura_tree();
                        let tree_count = tree.iter().map(|n| n.total_count()).sum();

                        ImportUpdate::Complete {
                            result,
                            tree,
                            tree_count,
                        }
                    }
                    Err(e) => ImportUpdate::Error(format!("Import failed: {}", e)),
                }
            },
            Message::ImportUpdate,
        )
    }

    /// Complete import with conflict resolutions (async)
    pub(crate) fn complete_import_with_resolutions_async(&mut self) -> Task<Message> {
        let Some(sv_path) = self.selected_sv_path.clone() else {
            self.set_error("No SavedVariables file selected");
            return Task::none();
        };

        let Some(conflict_result) = self.conflict_result.take() else {
            return Task::none();
        };

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

        self.is_importing = true;
        self.import_progress = 0.0;
        self.import_progress_message = "Starting import...".to_string();
        self.show_conflict_dialog = false;

        Task::perform(
            async move {
                let mut manager = SavedVariablesManager::new(sv_path);

                // Load existing
                if let Err(e) = manager.load() {
                    if !matches!(e, WeakAuraError::FileNotFound(_)) {
                        return ImportUpdate::Error(format!("Failed to load SavedVariables: {}", e));
                    }
                }

                // Apply resolutions
                let result = manager.apply_resolutions(
                    &conflict_result.new_auras,
                    &conflict_result.conflicts,
                    &resolutions,
                );

                // Save
                if let Err(e) = manager.save() {
                    return ImportUpdate::Error(format!("Failed to save: {}", e));
                }

                let tree = manager.get_aura_tree();
                let tree_count = tree.iter().map(|n| n.total_count()).sum();

                ImportUpdate::Complete {
                    result,
                    tree,
                    tree_count,
                }
            },
            Message::ImportUpdate,
        )
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

    /// Load from file dialog (async)
    pub(crate) fn load_from_file_async(&mut self) -> Task<Message> {
        Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Text files", &["txt", "md"])
                    .add_filter("All files", &["*"])
                    .pick_file()
                    .await
                    .map(|h| h.path().to_path_buf())
            },
            Message::FileSelected,
        )
    }

    /// Load file content after selection (async)
    pub(crate) fn load_file_content_async(&mut self, path: PathBuf) -> Task<Message> {
        let existing_ids = collect_existing_ids(&self.parsed_auras);

        self.is_loading = true;
        self.loading_progress = 0.0;
        self.loading_message = format!("Loading {}...", path.display());

        Task::perform(
            async move {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        let (entries, added, duplicates, invalid) =
                            decode_auras_filtered(&content, None, &existing_ids);

                        LoadingUpdate::Complete {
                            entries,
                            added,
                            duplicates,
                            invalid,
                        }
                    }
                    Err(e) => LoadingUpdate::Error(format!("Failed to read file: {}", e)),
                }
            },
            Message::LoadingUpdate,
        )
    }

    /// Load from folder dialog (async)
    pub(crate) fn load_from_folder_async(&mut self) -> Task<Message> {
        Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .pick_folder()
                    .await
                    .map(|h| h.path().to_path_buf())
            },
            Message::FolderSelected,
        )
    }

    /// Load folder content after selection (async)
    pub(crate) fn load_folder_content_async(&mut self, folder_path: PathBuf) -> Task<Message> {
        // Scan folder synchronously (fast filesystem walk)
        let file_paths = match Self::scan_folder_recursive(&folder_path) {
            Ok(paths) => paths,
            Err(e) => {
                self.set_error(&format!("Failed to scan folder: {}", e));
                return Task::none();
            }
        };

        if file_paths.is_empty() {
            self.set_status("No supported files found in folder");
            return Task::none();
        }

        let existing_ids = collect_existing_ids(&self.parsed_auras);

        self.is_loading = true;
        self.loading_progress = 0.0;
        self.loading_message = format!("Processing {} file(s)...", file_paths.len());

        Task::perform(
            async move {
                let mut all_entries = Vec::new();
                let mut total_added = 0;
                let mut total_duplicates = 0;
                let mut total_invalid = 0;
                let mut batch_ids = existing_ids;

                for file_path in file_paths {
                    let filename = file_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    let content = match tokio::fs::read_to_string(&file_path).await {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    let (entries, added, duplicates, invalid) =
                        decode_auras_filtered(&content, Some(filename), &batch_ids);

                    // Add newly discovered IDs to batch set for cross-file dedup
                    for entry in &entries {
                        if let Some(ref id) = entry.validation.aura_id {
                            batch_ids.insert(id.clone());
                        }
                    }

                    all_entries.extend(entries);
                    total_added += added;
                    total_duplicates += duplicates;
                    total_invalid += invalid;
                }

                LoadingUpdate::Complete {
                    entries: all_entries,
                    added: total_added,
                    duplicates: total_duplicates,
                    invalid: total_invalid,
                }
            },
            Message::LoadingUpdate,
        )
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

    /// Handle loading update from async task
    pub(crate) fn handle_loading_update(&mut self, update: LoadingUpdate) {
        match update {
            LoadingUpdate::Complete {
                entries,
                added,
                duplicates,
                invalid,
            } => {
                self.parsed_auras.extend(entries);
                self.is_loading = false;
                self.loading_progress = 1.0;
                self.loading_message.clear();

                let mut parts = Vec::new();
                parts.push(format!("{} added", added));
                if duplicates > 0 {
                    parts.push(format!("{} duplicates skipped", duplicates));
                }
                if invalid > 0 {
                    parts.push(format!("{} invalid", invalid));
                }
                self.set_status(&format!("Loaded: {}", parts.join(", ")));
            }
            LoadingUpdate::Error(msg) => {
                self.is_loading = false;
                self.loading_progress = 0.0;
                self.loading_message.clear();
                self.set_error(&msg);
            }
        }
    }

    /// Handle import update from async task
    pub(crate) fn handle_import_update(&mut self, update: ImportUpdate) {
        match update {
            ImportUpdate::ConflictsDetected(conflict_result) => {
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
            }
            ImportUpdate::Complete {
                result,
                tree,
                tree_count,
            } => {
                self.set_status(&format!("Import complete: {}", result.summary()));
                self.last_import_result = Some(result);
                self.existing_auras_tree = tree;
                self.existing_auras_count = tree_count;
                self.is_importing = false;
                self.import_progress = 1.0;
                self.import_progress_message = "Complete!".to_string();
                self.show_conflict_dialog = false;
                self.conflict_resolutions.clear();
            }
            ImportUpdate::Error(msg) => {
                self.set_error(&msg);
                self.is_importing = false;
                self.import_progress = 0.0;
                self.import_progress_message.clear();
            }
        }
    }

    /// Handle scan update from async task
    pub(crate) fn handle_scan_update(&mut self, update: ScanUpdate) {
        match update {
            ScanUpdate::Complete { tree, count } => {
                self.existing_auras_tree = tree;
                self.existing_auras_count = count;
                self.expanded_groups.clear();
                self.is_scanning = false;
                self.scanning_message.clear();
                self.set_status(&format!("Loaded {} existing aura(s)", count));
            }
            ScanUpdate::Error(msg) => {
                self.existing_auras_tree = Vec::new();
                self.existing_auras_count = 0;
                self.is_scanning = false;
                self.scanning_message.clear();
                self.set_error(&msg);
            }
        }
    }

    /// Handle removal update from async task
    pub(crate) fn handle_removal_update(&mut self, update: RemovalUpdate) {
        match update {
            RemovalUpdate::Complete {
                removed_count,
                tree,
                tree_count,
            } => {
                self.existing_auras_tree = tree;
                self.existing_auras_count = tree_count;
                self.selected_for_removal.clear();
                self.is_removing = false;
                self.removal_message.clear();
                if removed_count == 0 {
                    self.set_status("No auras were removed (already absent)");
                } else {
                    self.set_status(&format!("Removed {} aura(s)", removed_count));
                }
            }
            RemovalUpdate::Error(msg) => {
                self.is_removing = false;
                self.removal_message.clear();
                self.set_error(&msg);
            }
        }
    }

    pub(crate) fn set_status(&mut self, msg: &str) {
        self.status_message = msg.to_string();
        self.status_is_error = false;
    }

    pub(crate) fn set_error(&mut self, msg: &str) {
        self.status_message = msg.to_string();
        self.status_is_error = true;
    }

    /// Remove selected auras from SavedVariables (async)
    pub(crate) fn remove_auras_async(&mut self) -> Task<Message> {
        let Some(sv_path) = self.selected_sv_path.clone() else {
            self.set_error("No SavedVariables file selected");
            return Task::none();
        };

        let ids = std::mem::take(&mut self.pending_removal_ids);
        if ids.is_empty() {
            return Task::none();
        }

        self.is_removing = true;
        self.removal_message = "Removing auras...".to_string();

        Task::perform(
            async move {
                let mut manager = SavedVariablesManager::new(sv_path);

                if let Err(e) = manager.load() {
                    if !matches!(e, WeakAuraError::FileNotFound(_)) {
                        return RemovalUpdate::Error(format!(
                            "Failed to load SavedVariables: {}",
                            e
                        ));
                    }
                }

                let removed = manager.remove_auras(&ids);

                if removed.is_empty() {
                    let tree = manager.get_aura_tree();
                    let tree_count = tree.iter().map(|n| n.total_count()).sum();
                    return RemovalUpdate::Complete {
                        removed_count: 0,
                        tree,
                        tree_count,
                    };
                }

                if let Err(e) = manager.save() {
                    return RemovalUpdate::Error(format!("Failed to save: {}", e));
                }

                let tree = manager.get_aura_tree();
                let tree_count = tree.iter().map(|n| n.total_count()).sum();

                RemovalUpdate::Complete {
                    removed_count: removed.len(),
                    tree,
                    tree_count,
                }
            },
            Message::RemovalUpdate,
        )
    }
}
