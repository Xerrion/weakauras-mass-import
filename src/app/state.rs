//! Data types for GUI state that are shared across app submodules.

use std::collections::HashSet;

use crate::categories::UpdateCategory;
use crate::decoder::{ValidationResult, WeakAura};
use crate::saved_variables::ConflictAction;

/// Entry for a parsed aura in the list
pub(crate) struct ParsedAuraEntry {
    pub validation: ValidationResult,
    pub aura: Option<WeakAura>,
    pub selected: bool,
    /// Source file (if loaded from folder)
    pub source_file: Option<String>,
}

/// UI state for a single conflict resolution
#[derive(Clone)]
pub(crate) struct ConflictResolutionUI {
    /// The conflict this resolves
    pub aura_id: String,
    /// Current action
    pub action: ConflictAction,
    /// Categories to update (when action is UpdateSelected)
    pub categories: HashSet<UpdateCategory>,
    /// Whether to show details
    pub expanded: bool,
}
