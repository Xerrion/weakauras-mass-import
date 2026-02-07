//! Message types for the iced application.
//!
//! This module contains all messages that can be sent to update the application state.

use std::path::PathBuf;

use iced_toasts::ToastId;

use crate::categories::UpdateCategory;
use crate::saved_variables::ConflictAction;

use super::state::{ImportUpdate, LoadingUpdate, RemovalUpdate, ScanUpdate};

/// Messages for the iced application
#[derive(Debug, Clone)]
pub enum Message {
    // Input handling
    InputTextChanged(String),
    WowPathChanged(String),

    // File operations
    LoadFromFile,
    LoadFromFolder,
    BrowseWowPath,
    SelectSavedVariablesFile(PathBuf),
    SelectSavedVariablesManually,

    // Input actions
    TogglePasteInput,
    PasteFromClipboard,
    ParseInput,
    ClearInput,

    // View actions
    ToggleDecodedView,
    SelectAuraForPreview(usize),

    // Selection actions
    ToggleAuraSelection(usize),
    SelectAllAuras,
    DeselectAllAuras,
    RemoveAuraFromList(usize),
    RemoveSelectedFromList,

    // Import actions
    ShowImportConfirm,
    HideImportConfirm,
    ConfirmImport,

    // Conflict resolution
    HideConflictDialog,
    SetConflictAction(usize, ConflictAction),
    ToggleConflictExpanded(usize),
    ToggleGlobalCategory(UpdateCategory),
    ToggleConflictCategory(usize, UpdateCategory),
    SetAllConflictsAction(ConflictAction),
    ConfirmConflictResolutions,

    // Removal actions
    ToggleAuraForRemoval(String),
    SelectAllForRemoval,
    DeselectAllForRemoval,
    ShowRemoveConfirm,
    HideRemoveConfirm,
    ConfirmRemoval,

    // Setup wizard
    ShowSetupWizard,
    HideSetupWizard,

    // Tree navigation
    ToggleGroupExpanded(String),
    ExpandAllGroups,
    CollapseAllGroups,

    // Async task results
    LoadingUpdate(LoadingUpdate),
    ImportUpdate(ImportUpdate),
    ScanUpdate(ScanUpdate),
    RemovalUpdate(RemovalUpdate),

    // File dialog results
    FileSelected(Option<PathBuf>),
    FolderSelected(Option<PathBuf>),
    WowPathSelected(Option<PathBuf>),
    ManualSvSelected(Option<PathBuf>),

    // Sidebar resize
    StartSidebarResize,
    SidebarResize(f32),
    EndSidebarResize,
    HoverResizeEdge,
    UnhoverResizeEdge,

    // Toast notifications
    DismissToast(ToastId),
}
