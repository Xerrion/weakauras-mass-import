//! Business logic methods for WeakAuraImporter.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::decoder::{ValidationResult, WeakAura, WeakAuraDecoder};
use crate::error::WeakAuraError;
use crate::saved_variables::{ConflictAction, ConflictResolution, SavedVariablesManager};

use super::state::{ConflictResolutionUI, ParsedAuraEntry};
use super::WeakAuraImporter;

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

    /// Load existing auras from selected SavedVariables file (async)
    pub(crate) fn load_existing_auras(&mut self) {
        let Some(sv_path) = self.selected_sv_path.clone() else {
            return;
        };

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        self.is_scanning = true;
        self.scanning_message = "Loading SavedVariables...".to_string();
        self.scan_receiver = Some(rx);

        self.runtime.spawn(async move {
            let _ = tx
                .send(super::state::ScanUpdate::Progress {
                    message: "Reading SavedVariables file...".to_string(),
                })
                .await;

            let mut manager = SavedVariablesManager::new(sv_path);

            match manager.load() {
                Ok(()) => {
                    let tree = manager.get_aura_tree();
                    let count = tree.iter().map(|n| n.total_count()).sum();

                    let _ = tx
                        .send(super::state::ScanUpdate::Complete { tree, count })
                        .await;
                }
                Err(WeakAuraError::FileNotFound(_)) => {
                    // File doesn't exist yet — that's okay, just return empty
                    let _ = tx
                        .send(super::state::ScanUpdate::Complete {
                            tree: Vec::new(),
                            count: 0,
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(super::state::ScanUpdate::Error(format!(
                            "Failed to load SavedVariables: {}",
                            e
                        )))
                        .await;
                }
            }
        });
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

    /// Import selected auras to SavedVariables (async)
    pub(crate) fn import_auras(&mut self) {
        let Some(sv_path) = self.selected_sv_path.clone() else {
            self.set_error("No SavedVariables file selected");
            return;
        };

        // Collect selected valid auras on main thread before spawning
        let auras: Vec<WeakAura> = self
            .parsed_auras
            .iter()
            .filter(|e| e.selected && e.aura.is_some())
            .filter_map(|e| e.aura.clone())
            .collect();

        if auras.is_empty() {
            self.set_error("No valid auras selected for import");
            return;
        }

        // Clone global_categories for conflict resolution initialization
        let global_categories = self.global_categories.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        self.is_importing = true;
        self.import_progress = 0.0;
        self.import_progress_message = "Starting import...".to_string();
        self.import_receiver = Some(rx);

        self.runtime.spawn(async move {
            let _ = tx
                .send(super::state::ImportUpdate::Progress {
                    progress: 0.1,
                    message: "Loading SavedVariables...".to_string(),
                })
                .await;

            let mut manager = SavedVariablesManager::new(sv_path);

            // Load existing SavedVariables (blocking I/O in background task)
            if let Err(e) = manager.load() {
                if !matches!(e, WeakAuraError::FileNotFound(_)) {
                    let _ = tx
                        .send(super::state::ImportUpdate::Error(format!(
                            "Failed to load SavedVariables: {}",
                            e
                        )))
                        .await;
                    return;
                }
            }

            let _ = tx
                .send(super::state::ImportUpdate::Progress {
                    progress: 0.3,
                    message: "Detecting conflicts...".to_string(),
                })
                .await;

            // Detect conflicts
            let conflict_result = manager.detect_conflicts(&auras);

            // If there are conflicts, send back to UI for resolution
            if !conflict_result.conflicts.is_empty() {
                let _ = tx
                    .send(super::state::ImportUpdate::ConflictsDetected(
                        conflict_result,
                    ))
                    .await;
                return;
            }

            let _ = tx
                .send(super::state::ImportUpdate::Progress {
                    progress: 0.5,
                    message: "Importing auras...".to_string(),
                })
                .await;

            // No conflicts - import directly
            match manager.add_auras(&auras) {
                Ok(result) => {
                    let _ = tx
                        .send(super::state::ImportUpdate::Progress {
                            progress: 0.8,
                            message: "Saving changes...".to_string(),
                        })
                        .await;

                    if let Err(e) = manager.save() {
                        let _ = tx
                            .send(super::state::ImportUpdate::Error(format!(
                                "Failed to save: {}",
                                e
                            )))
                            .await;
                        return;
                    }

                    let tree = manager.get_aura_tree();
                    let tree_count = tree.iter().map(|n| n.total_count()).sum();

                    let _ = tx
                        .send(super::state::ImportUpdate::Complete {
                            result,
                            tree,
                            tree_count,
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(super::state::ImportUpdate::Error(format!(
                            "Import failed: {}",
                            e
                        )))
                        .await;
                }
            }

            // Keep global_categories alive until end of task (used by poll_importing for conflict init)
            drop(global_categories);
        });
    }

    /// Complete import with conflict resolutions (async)
    pub(crate) fn complete_import_with_resolutions(&mut self) {
        let Some(sv_path) = self.selected_sv_path.clone() else {
            self.set_error("No SavedVariables file selected");
            return;
        };

        let Some(conflict_result) = self.conflict_result.take() else {
            return;
        };

        // Convert UI resolutions to actual resolutions on main thread
        let resolutions: Vec<ConflictResolution> = self
            .conflict_resolutions
            .iter()
            .map(|r| ConflictResolution {
                aura_id: r.aura_id.clone(),
                action: r.action,
                categories_to_update: r.categories.clone(),
            })
            .collect();

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        self.is_importing = true;
        self.import_progress = 0.0;
        self.import_progress_message = "Starting import...".to_string();
        self.import_receiver = Some(rx);

        self.runtime.spawn(async move {
            let _ = tx
                .send(super::state::ImportUpdate::Progress {
                    progress: 0.1,
                    message: "Loading SavedVariables...".to_string(),
                })
                .await;

            let mut manager = SavedVariablesManager::new(sv_path);

            // Load existing
            if let Err(e) = manager.load() {
                if !matches!(e, WeakAuraError::FileNotFound(_)) {
                    let _ = tx
                        .send(super::state::ImportUpdate::Error(format!(
                            "Failed to load SavedVariables: {}",
                            e
                        )))
                        .await;
                    return;
                }
            }

            let _ = tx
                .send(super::state::ImportUpdate::Progress {
                    progress: 0.4,
                    message: "Applying resolutions...".to_string(),
                })
                .await;

            // Apply resolutions
            let result = manager.apply_resolutions(
                &conflict_result.new_auras,
                &conflict_result.conflicts,
                &resolutions,
            );

            let _ = tx
                .send(super::state::ImportUpdate::Progress {
                    progress: 0.7,
                    message: "Saving changes...".to_string(),
                })
                .await;

            // Save
            if let Err(e) = manager.save() {
                let _ = tx
                    .send(super::state::ImportUpdate::Error(format!(
                        "Failed to save: {}",
                        e
                    )))
                    .await;
                return;
            }

            let tree = manager.get_aura_tree();
            let tree_count = tree.iter().map(|n| n.total_count()).sum();

            let _ = tx
                .send(super::state::ImportUpdate::Complete {
                    result,
                    tree,
                    tree_count,
                })
                .await;
        });
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

    /// Load from file asynchronously, appending to existing auras with duplicate detection
    pub(crate) fn load_from_file_async(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt", "md"])
            .add_filter("All files", &["*"])
            .pick_file()
        else {
            return;
        };

        let existing_ids = collect_existing_ids(&self.parsed_auras);
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        self.is_loading = true;
        self.loading_progress = 0.0;
        self.loading_message = format!("Loading {}...", path.display());
        self.loading_receiver = Some(rx);

        self.runtime.spawn(async move {
            let _ = tx
                .send(super::state::LoadingUpdate::Progress {
                    current: 0,
                    total: 1,
                    message: format!("Reading {}", path.display()),
                })
                .await;

            match tokio::fs::read_to_string(&path).await {
                Ok(content) => {
                    let (entries, added, duplicates, invalid) =
                        decode_auras_filtered(&content, None, &existing_ids);

                    let _ = tx
                        .send(super::state::LoadingUpdate::Complete {
                            entries,
                            added,
                            duplicates,
                            invalid,
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(super::state::LoadingUpdate::Error(format!(
                            "Failed to read file: {}",
                            e
                        )))
                        .await;
                }
            }
        });
    }

    /// Load WeakAura strings from all files in a folder asynchronously,
    /// appending to existing auras with duplicate detection.
    pub(crate) fn load_from_folder_async(&mut self) {
        let Some(folder_path) = rfd::FileDialog::new().pick_folder() else {
            return;
        };

        // Scan folder synchronously (fast filesystem walk on main thread)
        let file_paths = match Self::scan_folder_recursive(&folder_path) {
            Ok(paths) => paths,
            Err(e) => {
                self.set_error(&format!("Failed to scan folder: {}", e));
                return;
            }
        };

        if file_paths.is_empty() {
            self.set_status("No supported files found in folder");
            return;
        }

        let existing_ids = collect_existing_ids(&self.parsed_auras);
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        self.is_loading = true;
        self.loading_progress = 0.0;
        self.loading_message = format!("Processing {} file(s)...", file_paths.len());
        self.loading_receiver = Some(rx);

        self.runtime.spawn(async move {
            let total_files = file_paths.len();
            let mut all_entries = Vec::new();
            let mut total_added = 0;
            let mut total_duplicates = 0;
            let mut total_invalid = 0;
            // Track IDs across the batch so files within the same load don't duplicate each other
            let mut batch_ids = existing_ids;

            for (i, file_path) in file_paths.iter().enumerate() {
                let filename = file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let _ = tx
                    .send(super::state::LoadingUpdate::Progress {
                        current: i,
                        total: total_files,
                        message: format!("Processing {} ({}/{})", filename, i + 1, total_files),
                    })
                    .await;

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

            let _ = tx
                .send(super::state::LoadingUpdate::Complete {
                    entries: all_entries,
                    added: total_added,
                    duplicates: total_duplicates,
                    invalid: total_invalid,
                })
                .await;
        });
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

    /// Poll for loading updates from background tasks.
    /// Called each frame from `update()`.
    pub(crate) fn poll_loading(&mut self) {
        let Some(rx) = &mut self.loading_receiver else {
            return;
        };

        // Drain all pending messages
        loop {
            match rx.try_recv() {
                Ok(super::state::LoadingUpdate::Progress {
                    current,
                    total,
                    message,
                }) => {
                    if total > 0 {
                        self.loading_progress = current as f32 / total as f32;
                    }
                    self.loading_message = message;
                }
                Ok(super::state::LoadingUpdate::Complete {
                    entries,
                    added,
                    duplicates,
                    invalid,
                }) => {
                    self.parsed_auras.extend(entries);
                    self.is_loading = false;
                    self.loading_receiver = None;
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
                    return; // Receiver was taken, stop loop
                }
                Ok(super::state::LoadingUpdate::Error(msg)) => {
                    self.is_loading = false;
                    self.loading_receiver = None;
                    self.loading_progress = 0.0;
                    self.loading_message.clear();
                    self.set_error(&msg);
                    return; // Receiver was taken, stop loop
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    // Sender dropped without sending Complete/Error
                    self.is_loading = false;
                    self.loading_receiver = None;
                    self.loading_progress = 0.0;
                    self.loading_message.clear();
                    return;
                }
            }
        }
    }

    /// Poll for import updates from background tasks.
    /// Called each frame from `update()`.
    pub(crate) fn poll_importing(&mut self) {
        let Some(rx) = &mut self.import_receiver else {
            return;
        };

        loop {
            match rx.try_recv() {
                Ok(super::state::ImportUpdate::Progress { progress, message }) => {
                    self.import_progress = progress;
                    self.import_progress_message = message;
                }
                Ok(super::state::ImportUpdate::ConflictsDetected(conflict_result)) => {
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
                    self.import_receiver = None;
                    return;
                }
                Ok(super::state::ImportUpdate::Complete {
                    result,
                    tree,
                    tree_count,
                }) => {
                    self.set_status(&format!("Import complete: {}", result.summary()));
                    self.last_import_result = Some(result);
                    self.existing_auras_tree = tree;
                    self.existing_auras_count = tree_count;
                    self.is_importing = false;
                    self.import_progress = 1.0;
                    self.import_progress_message = "Complete!".to_string();
                    self.import_receiver = None;
                    // Close conflict dialog if it was open
                    self.show_conflict_dialog = false;
                    self.conflict_resolutions.clear();
                    return;
                }
                Ok(super::state::ImportUpdate::Error(msg)) => {
                    self.set_error(&msg);
                    self.is_importing = false;
                    self.import_progress = 0.0;
                    self.import_progress_message.clear();
                    self.import_receiver = None;
                    return;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    self.is_importing = false;
                    self.import_receiver = None;
                    return;
                }
            }
        }
    }

    /// Poll for scan (SavedVariables loading) updates from background tasks.
    /// Called each frame from `update()`.
    pub(crate) fn poll_scanning(&mut self) {
        let Some(rx) = &mut self.scan_receiver else {
            return;
        };

        loop {
            match rx.try_recv() {
                Ok(super::state::ScanUpdate::Progress { message }) => {
                    self.scanning_message = message;
                }
                Ok(super::state::ScanUpdate::Complete { tree, count }) => {
                    self.existing_auras_tree = tree;
                    self.existing_auras_count = count;
                    self.expanded_groups.clear();
                    self.is_scanning = false;
                    self.scanning_message.clear();
                    self.scan_receiver = None;
                    self.set_status(&format!("Loaded {} existing aura(s)", count));
                    return;
                }
                Ok(super::state::ScanUpdate::Error(msg)) => {
                    self.existing_auras_tree = Vec::new();
                    self.existing_auras_count = 0;
                    self.is_scanning = false;
                    self.scanning_message.clear();
                    self.scan_receiver = None;
                    self.set_error(&msg);
                    return;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    self.is_scanning = false;
                    self.scan_receiver = None;
                    return;
                }
            }
        }
    }

    /// Poll for removal updates from background tasks.
    /// Called each frame from `update()`.
    pub(crate) fn poll_removing(&mut self) {
        let Some(rx) = &mut self.removal_receiver else {
            return;
        };

        loop {
            match rx.try_recv() {
                Ok(super::state::RemovalUpdate::Progress { message }) => {
                    self.removal_message = message;
                }
                Ok(super::state::RemovalUpdate::Complete {
                    removed_count,
                    tree,
                    tree_count,
                }) => {
                    self.existing_auras_tree = tree;
                    self.existing_auras_count = tree_count;
                    self.selected_for_removal.clear();
                    self.is_removing = false;
                    self.removal_message.clear();
                    self.removal_receiver = None;
                    if removed_count == 0 {
                        self.set_status("No auras were removed (already absent)");
                    } else {
                        self.set_status(&format!("Removed {} aura(s)", removed_count));
                    }
                    return;
                }
                Ok(super::state::RemovalUpdate::Error(msg)) => {
                    self.is_removing = false;
                    self.removal_message.clear();
                    self.removal_receiver = None;
                    self.set_error(&msg);
                    return;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    self.is_removing = false;
                    self.removal_receiver = None;
                    return;
                }
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

    /// Remove selected auras from SavedVariables (async, called after confirmation).
    pub(crate) fn remove_confirmed_auras(&mut self) {
        let Some(sv_path) = self.selected_sv_path.clone() else {
            self.set_error("No SavedVariables file selected");
            return;
        };

        let ids = std::mem::take(&mut self.pending_removal_ids);
        if ids.is_empty() {
            return;
        }

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        self.is_removing = true;
        self.removal_message = "Removing auras...".to_string();
        self.removal_receiver = Some(rx);

        self.runtime.spawn(async move {
            let _ = tx
                .send(super::state::RemovalUpdate::Progress {
                    message: "Loading SavedVariables...".to_string(),
                })
                .await;

            let mut manager = SavedVariablesManager::new(sv_path);

            if let Err(e) = manager.load() {
                if !matches!(e, WeakAuraError::FileNotFound(_)) {
                    let _ = tx
                        .send(super::state::RemovalUpdate::Error(format!(
                            "Failed to load SavedVariables: {}",
                            e
                        )))
                        .await;
                    return;
                }
            }

            let _ = tx
                .send(super::state::RemovalUpdate::Progress {
                    message: "Removing auras...".to_string(),
                })
                .await;

            let removed = manager.remove_auras(&ids);

            if removed.is_empty() {
                let tree = manager.get_aura_tree();
                let tree_count = tree.iter().map(|n| n.total_count()).sum();
                let _ = tx
                    .send(super::state::RemovalUpdate::Complete {
                        removed_count: 0,
                        tree,
                        tree_count,
                    })
                    .await;
                return;
            }

            let _ = tx
                .send(super::state::RemovalUpdate::Progress {
                    message: "Saving changes...".to_string(),
                })
                .await;

            if let Err(e) = manager.save() {
                let _ = tx
                    .send(super::state::RemovalUpdate::Error(format!(
                        "Failed to save: {}",
                        e
                    )))
                    .await;
                return;
            }

            let tree = manager.get_aura_tree();
            let tree_count = tree.iter().map(|n| n.total_count()).sum();

            let _ = tx
                .send(super::state::RemovalUpdate::Complete {
                    removed_count: removed.len(),
                    tree,
                    tree_count,
                })
                .await;
        });
    }
}
