//! Message update handlers for async task results.

use iced_toasts::{toast, ToastLevel};

use crate::saved_variables::ConflictAction;

use super::super::state::{
    ConflictResolutionUI, ImportUpdate, LoadingUpdate, RemovalUpdate, ScanUpdate,
};
use super::super::WeakAuraImporter;
use super::notify_decode_results;

impl WeakAuraImporter {
    /// Handle loading update from async task
    pub(crate) fn handle_loading_update(&mut self, update: LoadingUpdate) {
        match update {
            LoadingUpdate::Progress {
                current,
                total,
                message,
            } => {
                self.tasks.loading_progress = if total > 0 {
                    current as f32 / total as f32
                } else {
                    0.0
                };
                self.tasks.loading_message = message;
            }
            LoadingUpdate::Complete {
                entries,
                added,
                duplicates,
                errors,
            } => {
                self.parsed_auras.extend(entries);
                self.tasks.is_loading = false;
                self.tasks.loading_progress = 1.0;
                self.tasks.loading_message.clear();

                // Update status bar
                let total = self.parsed_auras.len();
                self.status.message = format!("{} aura(s) loaded, ready to import.", total);
                self.status.is_error = false;

                notify_decode_results(&mut self.toasts, added, duplicates, &errors, "loaded");
            }
            LoadingUpdate::Error(msg) => {
                self.tasks.is_loading = false;
                self.tasks.loading_progress = 0.0;
                self.tasks.loading_message.clear();
                self.status.message = format!("Load failed: {}", msg);
                self.status.is_error = true;
                self.toasts
                    .push(toast(&msg).title("Load Error").level(ToastLevel::Error));
            }
        }
    }

    /// Handle import update from async task
    pub(crate) fn handle_import_update(&mut self, update: ImportUpdate) {
        match update {
            ImportUpdate::Progress {
                current,
                total,
                message,
            } => {
                self.tasks.import_progress = if total > 0 {
                    current as f32 / total as f32
                } else {
                    0.0
                };
                self.tasks.import_message = message;
            }
            ImportUpdate::ConflictsDetected(conflict_result) => {
                // Initialize resolutions with defaults
                self.conflicts.resolutions = conflict_result
                    .conflicts
                    .iter()
                    .map(|c| ConflictResolutionUI {
                        aura_id: c.aura_id.clone(),
                        action: ConflictAction::UpdateSelected,
                        categories: self.conflicts.global_categories.clone(),
                        expanded: false,
                    })
                    .collect();
                self.conflicts.result = Some(conflict_result);
                self.ui.show_conflict_dialog = true;
                self.conflicts.selected_index = None;
                self.tasks.is_importing = false;
                self.tasks.import_progress = 0.0;
                self.tasks.import_message.clear();
            }
            ImportUpdate::Complete {
                result,
                tree,
                tree_count,
            } => {
                let summary = result.summary();
                self.status.message = format!("Import complete: {}", summary);
                self.status.is_error = false;
                self.toasts.push(
                    toast(&format!("Import complete: {}", summary))
                        .title("Success")
                        .level(ToastLevel::Success),
                );
                self.status.last_import_result = Some(result);
                self.saved_vars.auras_tree = tree;
                self.saved_vars.auras_count = tree_count;
                self.tasks.is_importing = false;
                self.tasks.import_progress = 1.0;
                self.tasks.import_message = "Complete!".to_string();
                self.ui.show_conflict_dialog = false;
                self.conflicts.resolutions.clear();
            }
            ImportUpdate::Error(msg) => {
                self.status.message = format!("Import failed: {}", msg);
                self.status.is_error = true;
                self.toasts
                    .push(toast(&msg).title("Import Error").level(ToastLevel::Error));
                self.tasks.is_importing = false;
                self.tasks.import_progress = 0.0;
                self.tasks.import_message.clear();
            }
        }
    }

    /// Handle scan update from async task
    pub(crate) fn handle_scan_update(&mut self, update: ScanUpdate) {
        match update {
            ScanUpdate::Complete { tree, count } => {
                self.saved_vars.auras_tree = tree;
                self.saved_vars.auras_count = count;
                self.sidebar.expanded_groups.clear();
                self.tasks.is_scanning = false;
                self.tasks.scanning_message.clear();
                if count > 0 {
                    self.status.message = format!("{} existing aura(s) in SavedVariables.", count);
                    self.status.is_error = false;
                    self.toasts.push(
                        toast(&format!("Loaded {} existing aura(s)", count))
                            .level(ToastLevel::Info),
                    );
                }
            }
            ScanUpdate::Error(msg) => {
                self.saved_vars.auras_tree = Vec::new();
                self.saved_vars.auras_count = 0;
                self.tasks.is_scanning = false;
                self.tasks.scanning_message.clear();
                self.status.message = format!("Scan failed: {}", msg);
                self.status.is_error = true;
                self.toasts
                    .push(toast(&msg).title("Scan Error").level(ToastLevel::Error));
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
                self.saved_vars.auras_tree = tree;
                self.saved_vars.auras_count = tree_count;
                self.removal.selected_ids.clear();
                self.tasks.is_removing = false;
                self.tasks.removal_message.clear();
                if removed_count == 0 {
                    self.status.message = "No auras removed (already absent).".to_string();
                    self.status.is_error = false;
                    self.toasts.push(
                        toast("No auras were removed (already absent)").level(ToastLevel::Info),
                    );
                } else {
                    self.status.message = format!("Removed {} aura(s).", removed_count);
                    self.status.is_error = false;
                    self.toasts.push(
                        toast(&format!("Removed {} aura(s)", removed_count))
                            .title("Success")
                            .level(ToastLevel::Success),
                    );
                }
            }
            RemovalUpdate::Error(msg) => {
                self.tasks.is_removing = false;
                self.tasks.removal_message.clear();
                self.status.message = format!("Removal failed: {}", msg);
                self.status.is_error = true;
                self.toasts
                    .push(toast(&msg).title("Removal Error").level(ToastLevel::Error));
            }
        }
    }
}
