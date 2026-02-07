//! Data types for GUI state that are shared across app submodules.

use std::collections::HashSet;

use crate::categories::UpdateCategory;
use crate::decoder::{ValidationResult, WeakAura};
use crate::saved_variables::{AuraTreeNode, ConflictAction, ConflictDetectionResult, ImportResult};

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
