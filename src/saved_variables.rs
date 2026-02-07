//! SavedVariables management for WeakAuras
//!
//! Handles reading and writing WeakAuras SavedVariables files.

use crate::categories::{CategoryMapper, UpdateCategory};
use crate::decoder::{LuaValue, WeakAura};
use crate::error::{Result, WeakAuraError};
use crate::lua_parser::LuaParser;
use crate::util;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;

/// Manages WeakAuras SavedVariables
pub struct SavedVariablesManager {
    /// Path to the SavedVariables file
    pub path: PathBuf,
    /// Loaded displays
    pub displays: HashMap<String, LuaValue>,
    /// Other fields (metadata like dbVersion, minimap, registered, etc.)
    other_fields: HashMap<String, LuaValue>,
    /// Raw file content for backup
    raw_content: Option<String>,
}

impl SavedVariablesManager {
    /// Create a new manager for a SavedVariables path
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            displays: HashMap::new(),
            other_fields: HashMap::new(),
            raw_content: None,
        }
    }

    /// Find WoW installation paths (common locations)
    pub fn find_wow_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "windows")]
        {
            // Common Windows installation paths
            let common_paths = [
                r"C:\Program Files (x86)\World of Warcraft",
                r"C:\Program Files\World of Warcraft",
                r"D:\World of Warcraft",
                r"E:\World of Warcraft",
                r"C:\Games\World of Warcraft",
                r"D:\Games\World of Warcraft",
            ];
            for path in &common_paths {
                let p = PathBuf::from(path);
                if p.exists() {
                    paths.push(p);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").unwrap_or_default();
            let common_paths = [
                "/Applications/World of Warcraft",
                &format!("{}/Applications/World of Warcraft", home),
            ];
            for path in &common_paths {
                let p = PathBuf::from(path);
                if p.exists() {
                    paths.push(p);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME").unwrap_or_default();
            // Wine/Lutris common paths
            let common_paths = [
                format!(
                    "{}/.wine/drive_c/Program Files (x86)/World of Warcraft",
                    home
                ),
                format!(
                    "{}/Games/world-of-warcraft/drive_c/Program Files (x86)/World of Warcraft",
                    home
                ),
            ];
            for path in common_paths {
                let p = PathBuf::from(&path);
                if p.exists() {
                    paths.push(p);
                }
            }
        }

        paths
    }

    /// Find SavedVariables files for all accounts
    pub fn find_saved_variables(wow_path: &Path) -> Vec<SavedVariablesInfo> {
        let mut results = Vec::new();

        // Check both retail and classic
        let flavors = [
            "_retail_",
            "_classic_",
            "_classic_era_",
            "_anniversary_",
            "_ptr_",
            "_beta_",
        ];

        for flavor in &flavors {
            let wtf_path = wow_path.join(flavor).join("WTF").join("Account");
            if wtf_path.exists() {
                if let Ok(accounts) = fs::read_dir(&wtf_path) {
                    for account in accounts.flatten() {
                        if account.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            let account_name = account.file_name().to_string_lossy().to_string();
                            let sv_path =
                                account.path().join("SavedVariables").join("WeakAuras.lua");
                            if sv_path.exists() {
                                results.push(SavedVariablesInfo {
                                    path: sv_path,
                                    account: account_name,
                                    flavor: flavor.trim_matches('_').to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        results
    }

    /// Load the SavedVariables file
    pub fn load(&mut self) -> Result<()> {
        if !self.path.exists() {
            return Err(WeakAuraError::FileNotFound(
                self.path.to_string_lossy().to_string(),
            ));
        }

        let content = fs::read_to_string(&self.path)?;
        self.raw_content = Some(content.clone());

        let saved = LuaParser::parse(&content)?;
        self.displays = saved.displays;
        self.other_fields = saved.other;

        Ok(())
    }

    /// Add multiple auras
    pub fn add_auras(&mut self, auras: &[WeakAura]) -> Result<ImportResult> {
        let mut added = Vec::new();
        let skipped = Vec::new();
        let mut replaced = Vec::new();

        for aura in auras {
            let hierarchy = util::build_children_hierarchy(aura);

            // Insert all prepared children into displays
            for (child_id, child_value) in &hierarchy.prepared_children {
                if self.displays.contains_key(child_id) {
                    replaced.push(child_id.clone());
                } else {
                    added.push(child_id.clone());
                }
                self.displays.insert(child_id.clone(), child_value.clone());
            }

            // Update subgroups' controlledChildren in the displays map
            // (build_children_hierarchy already set them on prepared_children,
            // but we re-inserted those above, so they're already correct)

            // Add the main aura (root parent) with its direct controlledChildren
            let mut parent_data = aura.data.clone();
            if let Some(direct_children) = hierarchy.children_by_parent.get(&aura.id) {
                util::set_controlled_children(&mut parent_data, direct_children);
            }

            if self.displays.contains_key(&aura.id) {
                replaced.push(aura.id.clone());
            } else {
                added.push(aura.id.clone());
            }
            self.displays.insert(aura.id.clone(), parent_data);
        }

        Ok(ImportResult {
            added,
            skipped,
            replaced,
        })
    }

    /// Detect conflicts between incoming auras and existing ones
    pub fn detect_conflicts(&self, auras: &[WeakAura]) -> ConflictDetectionResult {
        let mut result = ConflictDetectionResult::default();

        for aura in auras {
            let hierarchy = util::build_children_hierarchy(aura);

            // Prepare parent data with only direct controlledChildren
            let mut parent_data = aura.data.clone();
            if let Some(direct_children) = hierarchy.children_by_parent.get(&aura.id) {
                util::set_controlled_children(&mut parent_data, direct_children);
            }

            // Check main aura
            let total_child_count = hierarchy.prepared_children.len();
            if let Some(existing) = self.displays.get(&aura.id) {
                let conflict = ImportConflict::new(
                    aura.id.clone(),
                    parent_data.clone(),
                    existing.clone(),
                    aura.is_group,
                    total_child_count,
                );
                if conflict.has_changes() {
                    result.conflicts.push(conflict);
                }
            } else {
                result.new_auras.push((aura.id.clone(), parent_data));
            }

            // Check child auras
            for (child_id, child_value) in &hierarchy.prepared_children {
                let is_subgroup = hierarchy.children_by_parent.contains_key(child_id);
                let subgroup_child_count = if is_subgroup {
                    hierarchy
                        .children_by_parent
                        .get(child_id)
                        .map_or(0, |v| v.len())
                } else {
                    0
                };

                if let Some(existing) = self.displays.get(child_id) {
                    let conflict = ImportConflict::new(
                        child_id.clone(),
                        child_value.clone(),
                        existing.clone(),
                        is_subgroup,
                        subgroup_child_count,
                    );
                    if conflict.has_changes() {
                        result.conflicts.push(conflict);
                    }
                } else {
                    result
                        .new_auras
                        .push((child_id.clone(), child_value.clone()));
                }
            }
        }

        result
    }

    /// Perform selective merge based on category selection
    pub fn selective_merge(
        &mut self,
        conflict: &ImportConflict,
        categories: &HashSet<UpdateCategory>,
    ) {
        let Some(existing) = self.displays.get_mut(&conflict.aura_id) else {
            warn!(aura_id = %conflict.aura_id, "selective_merge: aura not found in displays");
            return;
        };

        let Some(incoming_table) = conflict.incoming.as_table() else {
            warn!(aura_id = %conflict.aura_id, "selective_merge: incoming data is not a table variant");
            return;
        };
        let Some(existing_table) = existing.as_table_mut() else {
            warn!(aura_id = %conflict.aura_id, "selective_merge: existing data is not a table variant");
            return;
        };

        // For each category that should be updated
        for category in categories {
            let fields = CategoryMapper::get_fields(*category);

            if *category == UpdateCategory::Display {
                // Display is a catch-all - copy all fields not in other categories
                for (field, value) in incoming_table {
                    if CategoryMapper::is_internal_field(field) {
                        continue;
                    }
                    if CategoryMapper::get_category(field) == UpdateCategory::Display {
                        existing_table.insert(field.clone(), value.clone());
                    }
                }
            } else {
                // Copy specific fields for this category
                for field in fields {
                    if let Some(value) = incoming_table.get(*field) {
                        existing_table.insert(field.to_string(), value.clone());
                    } else {
                        // Field exists in existing but not incoming - remove it
                        existing_table.remove(*field);
                    }
                }
            }
        }
    }

    /// Apply all resolutions (convenience method)
    pub fn apply_resolutions(
        &mut self,
        new_auras: &[(String, LuaValue)],
        conflicts: &[ImportConflict],
        resolutions: &[ConflictResolution],
    ) -> ImportResult {
        let mut added = Vec::new();
        let mut skipped = Vec::new();
        let mut replaced = Vec::new();

        // Add all new auras
        for (id, data) in new_auras {
            self.displays.insert(id.clone(), data.clone());
            added.push(id.clone());
        }

        // Create a map for quick lookup
        let conflict_map: HashMap<&str, &ImportConflict> =
            conflicts.iter().map(|c| (c.aura_id.as_str(), c)).collect();

        // Apply resolutions
        for resolution in resolutions {
            match resolution.action {
                ConflictAction::Skip => {
                    skipped.push(resolution.aura_id.clone());
                }
                ConflictAction::ReplaceAll => {
                    if let Some(conflict) = conflict_map.get(resolution.aura_id.as_str()) {
                        self.displays
                            .insert(resolution.aura_id.clone(), conflict.incoming.clone());
                        replaced.push(resolution.aura_id.clone());
                    }
                }
                ConflictAction::UpdateSelected => {
                    if let Some(conflict) = conflict_map.get(resolution.aura_id.as_str()) {
                        self.selective_merge(conflict, &resolution.categories_to_update);
                        replaced.push(resolution.aura_id.clone());
                    }
                }
            }
        }

        ImportResult {
            added,
            skipped,
            replaced,
        }
    }

    /// Get auras organized in a tree structure (groups with children)
    pub fn get_aura_tree(&self) -> Vec<AuraTreeNode> {
        let mut children_map: HashMap<String, Vec<String>> = HashMap::new();

        // First pass: identify all parent-child relationships
        for (id, data) in &self.displays {
            if let Some(table) = data.as_table() {
                if let Some(LuaValue::String(parent_id)) = table.get("parent") {
                    children_map
                        .entry(parent_id.clone())
                        .or_default()
                        .push(id.clone());
                }
            }
        }

        // Recursive helper to build a tree node
        fn build_node(
            id: &str,
            displays: &HashMap<String, LuaValue>,
            children_map: &HashMap<String, Vec<String>>,
        ) -> AuraTreeNode {
            let is_group = displays
                .get(id)
                .and_then(|d| d.as_table())
                .map(|t| {
                    matches!(
                        t.get("regionType"),
                        Some(LuaValue::String(rt)) if rt == "group" || rt == "dynamicgroup"
                    )
                })
                .unwrap_or(false);

            let children = if is_group {
                children_map
                    .get(id)
                    .map(|child_ids| {
                        let mut children: Vec<AuraTreeNode> = child_ids
                            .iter()
                            .map(|child_id| build_node(child_id, displays, children_map))
                            .collect();
                        children.sort_by_key(|a| a.id.to_lowercase());
                        children
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            AuraTreeNode {
                id: id.to_string(),
                is_group,
                children,
            }
        }

        // Build tree nodes for top-level auras (no parent)
        let mut nodes: Vec<AuraTreeNode> = self
            .displays
            .iter()
            .filter(|(_, data)| {
                data.as_table()
                    .map(|t| t.get("parent").is_none())
                    .unwrap_or(false)
            })
            .map(|(id, _)| build_node(id, &self.displays, &children_map))
            .collect();

        // Sort top-level nodes: groups first, then alphabetically
        nodes.sort_by(|a, b| match (a.is_group, b.is_group) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.id.to_lowercase().cmp(&b.id.to_lowercase()),
        });

        nodes
    }

    /// Remove auras by ID, recursively removing children of groups.
    ///
    /// For each ID:
    /// - If it's a group, all descendants are also removed.
    /// - If it has a parent, the parent's `controlledChildren` is updated.
    ///
    /// Returns the list of all IDs that were actually removed.
    pub fn remove_auras(&mut self, ids: &[String]) -> Vec<String> {
        let mut removed = Vec::new();

        for id in ids {
            // Collect this aura and all its descendants
            let to_remove = self.collect_descendants(id);

            // Find the parent of the top-level aura being removed
            let parent_id = self
                .displays
                .get(id)
                .and_then(|d| d.as_table())
                .and_then(|t| t.get("parent"))
                .and_then(|v| {
                    if let LuaValue::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                });

            // Remove all collected auras from displays
            for remove_id in &to_remove {
                if self.displays.remove(remove_id).is_some() {
                    removed.push(remove_id.clone());
                }
            }

            // Update parent's controlledChildren if applicable
            if let Some(parent_id) = parent_id {
                if let Some(parent_data) = self.displays.get_mut(&parent_id) {
                    if let Some(table) = parent_data.as_table_mut() {
                        if let Some(LuaValue::Array(children)) = table.get_mut("controlledChildren")
                        {
                            children.retain(|v| {
                                if let LuaValue::String(child_id) = v {
                                    child_id != id
                                } else {
                                    true
                                }
                            });
                        }
                    }
                }
            }
        }

        removed
    }

    /// Collect an aura ID and all its descendant IDs (recursive).
    fn collect_descendants(&self, id: &str) -> Vec<String> {
        let mut result = vec![id.to_string()];

        // Check if this aura has controlledChildren
        if let Some(data) = self.displays.get(id) {
            if let Some(table) = data.as_table() {
                if let Some(LuaValue::Array(children)) = table.get("controlledChildren") {
                    for child in children {
                        if let LuaValue::String(child_id) = child {
                            result.extend(self.collect_descendants(child_id));
                        }
                    }
                }
            }
        }

        result
    }

    /// Save the SavedVariables back to file
    pub fn save(&self) -> Result<()> {
        // Create backup first
        if self.path.exists() {
            let backup_path = self.path.with_extension("lua.backup");
            fs::copy(&self.path, &backup_path)?;
        }

        // Generate new content
        let content = self.generate_lua();
        fs::write(&self.path, content)?;

        Ok(())
    }

    /// Generate Lua content for SavedVariables
    pub fn generate_lua(&self) -> String {
        let mut output = String::new();
        output.push_str("\nWeakAurasSaved = {\n");

        // Write other fields first (metadata like dbVersion, minimap, registered, etc.)
        let mut other_keys: Vec<_> = self.other_fields.keys().collect();
        other_keys.sort();
        for key in other_keys {
            let value = &self.other_fields[key];
            let escaped_key = util::escape_lua_string(key);
            output.push_str(&format!(
                "\t[\"{}\"] = {},\n",
                escaped_key,
                LuaParser::serialize(value, 1)
            ));
        }

        // Write displays
        output.push_str("\t[\"displays\"] = {\n");
        let mut display_keys: Vec<_> = self.displays.keys().collect();
        display_keys.sort();
        for id in display_keys {
            let data = &self.displays[id];
            // Escape special characters in the ID
            let escaped_id = util::escape_lua_string(id);
            output.push_str(&format!(
                "\t\t[\"{}\"] = {},\n",
                escaped_id,
                LuaParser::serialize(data, 2)
            ));
        }
        output.push_str("\t},\n");

        output.push_str("}\n");
        output
    }
}

/// Information about a found SavedVariables file
#[derive(Debug, Clone)]
pub struct SavedVariablesInfo {
    pub path: PathBuf,
    pub account: String,
    pub flavor: String,
}

impl SavedVariablesInfo {
    /// Returns a pretty-formatted flavor name (e.g., "classic_era" → "Classic Era")
    pub fn pretty_flavor(&self) -> String {
        format_flavor_name(&self.flavor)
    }
}

impl std::fmt::Display for SavedVariablesInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} ({})",
            self.account,
            self.pretty_flavor(),
            self.path.display()
        )
    }
}

/// Format a WoW flavor name to a pretty display name.
/// E.g., "classic" → "Classic", "classic_era" → "Classic Era", "retail" → "Retail"
pub fn format_flavor_name(flavor: &str) -> String {
    flavor
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A node in the aura tree (for hierarchical display)
#[derive(Debug, Clone)]
pub struct AuraTreeNode {
    /// Aura ID/name
    pub id: String,
    /// Whether this is a group
    pub is_group: bool,
    /// Child auras (if this is a group)
    pub children: Vec<AuraTreeNode>,
}

impl AuraTreeNode {
    /// Count total auras (including children)
    pub fn total_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.total_count()).sum::<usize>()
    }
}

/// Result of importing auras
#[derive(Debug, Default, Clone)]
pub struct ImportResult {
    pub added: Vec<String>,
    pub skipped: Vec<String>,
    pub replaced: Vec<String>,
}

impl ImportResult {
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.added.is_empty() {
            parts.push(format!("{} added", self.added.len()));
        }
        if !self.replaced.is_empty() {
            parts.push(format!("{} replaced", self.replaced.len()));
        }
        if !self.skipped.is_empty() {
            parts.push(format!("{} skipped", self.skipped.len()));
        }
        if parts.is_empty() {
            "No changes".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Represents a conflict between an incoming aura and an existing one
#[derive(Debug, Clone)]
pub struct ImportConflict {
    /// ID of the conflicting aura
    pub aura_id: String,
    /// The incoming (new) aura data
    pub incoming: LuaValue,
    /// Categories that have differences
    pub changed_categories: HashSet<UpdateCategory>,
    /// Whether this is a group
    pub is_group: bool,
    /// Child count (if group)
    pub child_count: usize,
}

impl ImportConflict {
    /// Create a new conflict
    pub fn new(
        aura_id: String,
        incoming: LuaValue,
        existing: LuaValue,
        is_group: bool,
        child_count: usize,
    ) -> Self {
        let changed_categories = Self::detect_changed_categories(&incoming, &existing);
        Self {
            aura_id,
            incoming,
            changed_categories,
            is_group,
            child_count,
        }
    }

    /// Detect which categories have changes between incoming and existing
    fn detect_changed_categories(
        incoming: &LuaValue,
        existing: &LuaValue,
    ) -> HashSet<UpdateCategory> {
        let mut changed = HashSet::new();

        let (Some(incoming_table), Some(existing_table)) =
            (incoming.as_table(), existing.as_table())
        else {
            warn!("detect_changed_categories: one or both values are not table variants");
            return changed;
        };

        // Check all fields in incoming
        for (field, incoming_value) in incoming_table {
            if CategoryMapper::is_internal_field(field) {
                continue;
            }

            let category = CategoryMapper::get_category(field);

            // Compare values
            let existing_value = existing_table.get(field);
            if existing_value != Some(incoming_value) {
                changed.insert(category);
            }
        }

        // Check for fields in existing but not in incoming (would be removed)
        for field in existing_table.keys() {
            if CategoryMapper::is_internal_field(field) {
                continue;
            }

            if !incoming_table.contains_key(field) {
                let category = CategoryMapper::get_category(field);
                changed.insert(category);
            }
        }

        changed
    }

    /// Check if any category has changes
    pub fn has_changes(&self) -> bool {
        !self.changed_categories.is_empty()
    }
}

/// Resolution for a single conflict
#[derive(Debug, Clone)]
pub struct ConflictResolution {
    /// Aura ID
    pub aura_id: String,
    /// Action to take
    pub action: ConflictAction,
    /// Categories to update (only used when action is UpdateSelected)
    pub categories_to_update: HashSet<UpdateCategory>,
}

/// Action for resolving a conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictAction {
    /// Skip this aura (keep existing)
    Skip,
    /// Replace entirely with incoming
    ReplaceAll,
    /// Update only selected categories
    UpdateSelected,
}

impl std::fmt::Display for ConflictAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictAction::Skip => write!(f, "Skip"),
            ConflictAction::ReplaceAll => write!(f, "Replace"),
            ConflictAction::UpdateSelected => write!(f, "Update"),
        }
    }
}

impl Default for ConflictResolution {
    fn default() -> Self {
        Self {
            aura_id: String::new(),
            action: ConflictAction::UpdateSelected,
            categories_to_update: UpdateCategory::defaults(),
        }
    }
}

/// Result of conflict detection
#[derive(Debug, Default, Clone)]
pub struct ConflictDetectionResult {
    /// Auras that don't exist (no conflict)
    pub new_auras: Vec<(String, LuaValue)>,
    /// Auras that have conflicts
    pub conflicts: Vec<ImportConflict>,
}
