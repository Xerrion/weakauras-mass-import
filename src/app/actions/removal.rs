//! Remove auras from SavedVariables and scan existing auras.

use iced::Task;
use iced_toasts::{toast, ToastLevel};

use crate::error::WeakAuraError;
use crate::saved_variables::SavedVariablesManager;

use super::super::state::{RemovalUpdate, ScanUpdate};
use super::super::{Message, WeakAuraImporter};

impl WeakAuraImporter {
    /// Load existing auras from selected SavedVariables file (async)
    pub(crate) fn load_existing_auras_async(&mut self) -> Task<Message> {
        let Some(sv_path) = self.saved_vars.selected_path.clone() else {
            return Task::none();
        };

        self.tasks.is_scanning = true;
        self.tasks.scanning_message = "Loading SavedVariables...".to_string();

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

    /// Remove selected auras from SavedVariables (async)
    pub(crate) fn remove_auras_async(&mut self) -> Task<Message> {
        let Some(sv_path) = self.saved_vars.selected_path.clone() else {
            self.toasts.push(
                toast("No SavedVariables file selected")
                    .title("Removal Error")
                    .level(ToastLevel::Error),
            );
            return Task::none();
        };

        let ids = std::mem::take(&mut self.removal.pending_ids);
        if ids.is_empty() {
            return Task::none();
        }

        self.tasks.is_removing = true;
        self.tasks.removal_message = "Removing auras...".to_string();

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
