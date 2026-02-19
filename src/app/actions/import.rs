//! Import auras to SavedVariables (with conflict detection and resolution).

use std::path::PathBuf;

use iced::futures::SinkExt;
use iced::{stream, Task};
use iced_toasts::{toast, ToastLevel};

use crate::decoder::WeakAura;
use crate::error::WeakAuraError;
use crate::saved_variables::{ConflictResolution, SavedVariablesManager};

use super::super::state::ImportUpdate;
use super::super::{Message, WeakAuraImporter};

impl WeakAuraImporter {
    /// Import selected auras to SavedVariables (async with streaming progress)
    pub(crate) fn import_auras_async(&mut self) -> Task<Message> {
        let Some(sv_path) = self.selected_sv_path.clone() else {
            self.toasts.push(
                toast("No SavedVariables file selected")
                    .title("Import Error")
                    .level(ToastLevel::Error),
            );
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
            self.toasts.push(
                toast("No valid auras selected for import")
                    .title("Import Error")
                    .level(ToastLevel::Error),
            );
            return Task::none();
        }

        self.is_importing = true;
        self.import_progress = 0.0;
        self.import_progress_message = "Starting import...".to_string();

        Task::run(
            stream::channel(
                100,
                move |mut sender: iced::futures::channel::mpsc::Sender<Message>| async move {
                    run_import_pipeline(sv_path, auras, &mut sender).await;
                },
            ),
            |msg| msg,
        )
    }

    /// Complete import with conflict resolutions (async with streaming progress)
    pub(crate) fn complete_import_with_resolutions_async(&mut self) -> Task<Message> {
        let Some(sv_path) = self.selected_sv_path.clone() else {
            self.toasts.push(
                toast("No SavedVariables file selected")
                    .title("Import Error")
                    .level(ToastLevel::Error),
            );
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

        Task::run(
            stream::channel(
                100,
                move |mut sender: iced::futures::channel::mpsc::Sender<Message>| async move {
                    run_import_with_resolutions(sv_path, conflict_result, resolutions, &mut sender)
                        .await;
                },
            ),
            |msg| msg,
        )
    }
}

/// Send a progress update message
async fn send_progress(
    sender: &mut iced::futures::channel::mpsc::Sender<Message>,
    current: usize,
    total: usize,
    message: impl Into<String>,
) {
    let _ = sender
        .send(Message::ImportUpdate(ImportUpdate::Progress {
            current,
            total,
            message: message.into(),
        }))
        .await;
}

/// Send an error message
async fn send_error(sender: &mut iced::futures::channel::mpsc::Sender<Message>, msg: String) {
    let _ = sender
        .send(Message::ImportUpdate(ImportUpdate::Error(msg)))
        .await;
}

/// Load SavedVariables manager, handling the common "file not found is OK" pattern
async fn load_manager(
    sv_path: PathBuf,
    sender: &mut iced::futures::channel::mpsc::Sender<Message>,
) -> Option<SavedVariablesManager> {
    let mut manager = SavedVariablesManager::new(sv_path);
    if let Err(e) = manager.load() {
        if !matches!(e, WeakAuraError::FileNotFound(_)) {
            send_error(sender, format!("Failed to load SavedVariables: {}", e)).await;
            return None;
        }
    }
    Some(manager)
}

/// Run the import pipeline (used by import_auras_async)
async fn run_import_pipeline(
    sv_path: PathBuf,
    auras: Vec<WeakAura>,
    sender: &mut iced::futures::channel::mpsc::Sender<Message>,
) {
    // Phase 1: Loading SavedVariables (0-25%)
    send_progress(sender, 1, 4, "Loading SavedVariables...").await;

    let Some(mut manager) = load_manager(sv_path, sender).await else {
        return;
    };

    // Phase 2: Detecting conflicts (25-50%)
    send_progress(sender, 2, 4, "Detecting conflicts...").await;

    let conflict_result = manager.detect_conflicts(&auras);

    // If there are conflicts, send back to UI for resolution
    if !conflict_result.conflicts.is_empty() {
        let _ = sender
            .send(Message::ImportUpdate(ImportUpdate::ConflictsDetected(
                conflict_result,
            )))
            .await;
        return;
    }

    // Phase 3: Importing auras (50-75%)
    send_progress(
        sender,
        3,
        4,
        format!("Importing {} aura(s)...", auras.len()),
    )
    .await;

    let result = match manager.add_auras(&auras) {
        Ok(r) => r,
        Err(e) => {
            send_error(sender, format!("Import failed: {}", e)).await;
            return;
        }
    };

    // Phase 4: Saving (75-100%)
    send_progress(sender, 4, 4, "Saving changes...").await;

    if let Err(e) = manager.save() {
        send_error(sender, format!("Failed to save: {}", e)).await;
        return;
    }

    let tree = manager.get_aura_tree();
    let tree_count = tree.iter().map(|n| n.total_count()).sum();

    let _ = sender
        .send(Message::ImportUpdate(ImportUpdate::Complete {
            result,
            tree,
            tree_count,
        }))
        .await;
}

/// Run import with conflict resolutions (used by complete_import_with_resolutions_async)
async fn run_import_with_resolutions(
    sv_path: PathBuf,
    conflict_result: crate::saved_variables::ConflictDetectionResult,
    resolutions: Vec<ConflictResolution>,
    sender: &mut iced::futures::channel::mpsc::Sender<Message>,
) {
    // Phase 1: Loading SavedVariables (0-33%)
    send_progress(sender, 1, 3, "Loading SavedVariables...").await;

    let Some(mut manager) = load_manager(sv_path, sender).await else {
        return;
    };

    // Phase 2: Applying resolutions (33-66%)
    send_progress(sender, 2, 3, "Applying conflict resolutions...").await;

    let result = manager.apply_resolutions(&conflict_result, &resolutions);

    // Phase 3: Saving (66-100%)
    send_progress(sender, 3, 3, "Saving changes...").await;

    if let Err(e) = manager.save() {
        send_error(sender, format!("Failed to save: {}", e)).await;
        return;
    }

    let tree = manager.get_aura_tree();
    let tree_count = tree.iter().map(|n| n.total_count()).sum();

    let _ = sender
        .send(Message::ImportUpdate(ImportUpdate::Complete {
            result,
            tree,
            tree_count,
        }))
        .await;
}
