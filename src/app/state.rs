//! Data types for GUI state that are shared across app submodules.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::categories::UpdateCategory;
use crate::decoder::{ValidationResult, WeakAura};
use crate::saved_variables::{
    AuraTreeNode, ConflictAction, ConflictDetectionResult, ImportResult, SavedVariablesInfo,
};

// =============================================================================
// Nested State Structs for WeakAuraImporter
// =============================================================================

/// UI visibility flags (which panels/dialogs are shown)
#[derive(Debug, Default)]
pub struct UiVisibility {
    /// Show decoded view panel
    pub show_decoded_view: bool,
    /// Show paste input area
    pub show_paste_input: bool,
    /// Show import confirmation dialog
    pub show_import_confirm: bool,
    /// Show conflict resolution dialog
    pub show_conflict_dialog: bool,
    /// Show removal confirmation dialog
    pub show_remove_confirm: bool,
    /// Show setup wizard for selecting SavedVariables
    pub show_setup_wizard: bool,
}

/// Sidebar-related state
#[derive(Debug)]
pub struct SidebarState {
    /// Sidebar width in pixels
    pub width: f32,
    /// Whether sidebar is being resized
    pub is_resizing: bool,
    /// Whether mouse is hovering over resize edge
    pub is_hovering_resize: bool,
    /// Expanded groups in the existing auras tree
    pub expanded_groups: HashSet<String>,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            width: 350.0,
            is_resizing: false,
            is_hovering_resize: false,
            expanded_groups: HashSet::new(),
        }
    }
}

/// Background task progress state
#[derive(Debug, Default)]
pub struct TaskProgress {
    /// Whether a background loading task is in progress
    pub is_loading: bool,
    /// Loading progress (0.0 to 1.0)
    pub loading_progress: f32,
    /// Loading progress message
    pub loading_message: String,
    /// Whether a background scanning task is in progress
    pub is_scanning: bool,
    /// Scanning progress message
    pub scanning_message: String,
    /// Whether a background import task is in progress
    pub is_importing: bool,
    /// Import progress (0.0 to 1.0)
    pub import_progress: f32,
    /// Import progress message
    pub import_message: String,
    /// Whether a background removal task is in progress
    pub is_removing: bool,
    /// Removal progress message
    pub removal_message: String,
}

/// Conflict resolution state
#[derive(Debug)]
pub struct ConflictState {
    /// Current conflict detection result
    pub result: Option<ConflictDetectionResult>,
    /// Resolutions for each conflict (parallel to conflict_result.conflicts)
    pub resolutions: Vec<ConflictResolutionUI>,
    /// Global categories to update (used as default for all conflicts)
    pub global_categories: HashSet<UpdateCategory>,
    /// Selected conflict index in the dialog
    pub selected_index: Option<usize>,
}

impl Default for ConflictState {
    fn default() -> Self {
        Self {
            result: None,
            resolutions: Vec::new(),
            global_categories: UpdateCategory::defaults(),
            selected_index: None,
        }
    }
}

/// Aura removal state
#[derive(Debug, Default)]
pub struct RemovalState {
    /// Auras selected for removal in the sidebar
    pub selected_ids: HashSet<String>,
    /// IDs pending removal (populated when confirm dialog opens)
    pub pending_ids: Vec<String>,
}

/// SavedVariables file management state
#[derive(Debug, Default)]
pub struct SavedVariablesState {
    /// WoW path for scanning
    pub wow_path: String,
    /// Selected SavedVariables file
    pub selected_path: Option<PathBuf>,
    /// Discovered SavedVariables files
    pub discovered_files: Vec<SavedVariablesInfo>,
    /// Loaded existing auras as tree
    pub auras_tree: Vec<AuraTreeNode>,
    /// Total count of existing auras
    pub auras_count: usize,
}

/// Status bar state
#[derive(Debug)]
pub struct StatusState {
    /// Status message
    pub message: String,
    /// Status is error
    pub is_error: bool,
    /// Last import result
    pub last_import_result: Option<ImportResult>,
}

impl Default for StatusState {
    fn default() -> Self {
        Self {
            message: String::from("Ready. Load WeakAura strings from file or folder."),
            is_error: false,
            last_import_result: None,
        }
    }
}

// =============================================================================
// Original Types (for async task results and parsed entries)
// =============================================================================

/// Entry for a parsed aura in the list
#[derive(Clone, Debug)]
pub struct ParsedAuraEntry {
    pub validation: ValidationResult,
    pub aura: Option<WeakAura>,
    pub selected: bool,
}

/// UI state for a single conflict resolution
#[derive(Clone, Debug)]
pub struct ConflictResolutionUI {
    /// The conflict this resolves
    pub aura_id: String,
    /// Current action
    pub action: ConflictAction,
    /// Categories to update (when action is UpdateSelected)
    pub categories: HashSet<UpdateCategory>,
    /// Whether to show details
    pub expanded: bool,
}

/// Result from background loading task
#[derive(Clone, Debug)]
pub enum LoadingUpdate {
    /// Progress update during loading (0.0 to 1.0)
    Progress {
        current: usize,
        total: usize,
        message: String,
    },
    /// Loading completed successfully
    Complete {
        entries: Vec<ParsedAuraEntry>,
        added: usize,
        duplicates: usize,
        errors: Vec<String>,
    },
    /// Loading failed with an error
    Error(String),
}

/// Result from background import task
#[derive(Clone, Debug)]
pub enum ImportUpdate {
    /// Progress update during import (0.0 to 1.0)
    #[allow(dead_code)]
    Progress {
        current: usize,
        total: usize,
        message: String,
    },
    /// Conflicts detected — hand data back to UI for resolution
    ConflictsDetected(ConflictDetectionResult),
    /// Import completed successfully
    Complete {
        result: ImportResult,
        tree: Vec<AuraTreeNode>,
        tree_count: usize,
    },
    /// Import failed with an error
    Error(String),
}

/// Result from background SavedVariables scanning task
#[derive(Clone, Debug)]
pub enum ScanUpdate {
    /// Scanning completed successfully
    Complete {
        tree: Vec<AuraTreeNode>,
        count: usize,
    },
    /// Scanning failed with an error
    Error(String),
}

/// Result from background aura removal task
#[derive(Clone, Debug)]
pub enum RemovalUpdate {
    /// Removal completed successfully
    Complete {
        removed_count: usize,
        tree: Vec<AuraTreeNode>,
        tree_count: usize,
    },
    /// Removal failed with an error
    Error(String),
}
