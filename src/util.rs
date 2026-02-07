//! Shared utility functions for the WeakAura importer

use crate::decoder::{LuaValue, WeakAura};
use std::collections::HashMap;

/// Escape special characters in Lua strings
pub fn escape_lua_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Result of building a children hierarchy from a WeakAura's flat child_data.
///
/// Contains the parent->children ID mapping and the prepared child data
/// (with `parent` fields ensured and `controlledChildren` set on subgroups).
pub struct ChildrenHierarchy {
    /// Map of parent_id -> ordered list of direct child IDs
    pub children_by_parent: HashMap<String, Vec<String>>,
    /// Map of child_id -> prepared child LuaValue (with parent field set)
    pub prepared_children: HashMap<String, LuaValue>,
}

/// Build a children hierarchy from a WeakAura's flat child_data list.
///
/// This extracts parent->children relationships from each child's `parent` field,
/// ensures every child has a `parent` field (falling back to the root aura ID),
/// and sets `controlledChildren` on subgroups.
pub fn build_children_hierarchy(aura: &WeakAura) -> ChildrenHierarchy {
    let mut children_by_parent: HashMap<String, Vec<String>> = HashMap::new();
    let mut prepared_children: HashMap<String, LuaValue> = HashMap::new();

    // First pass: extract parent->child relationships and prepare child data
    for child_data in &aura.child_data {
        if let Some(child_table) = child_data.as_table() {
            if let Some(LuaValue::String(child_id)) = child_table.get("id") {
                // Determine the parent: use existing parent field, fall back to root aura ID
                let parent_id = child_table
                    .get("parent")
                    .and_then(|v| {
                        if let LuaValue::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| aura.id.clone());

                children_by_parent
                    .entry(parent_id)
                    .or_default()
                    .push(child_id.clone());

                // Prepare child data, ensuring parent field is set
                let mut child_value = child_data.clone();
                if let Some(child_table_mut) = child_value.as_table_mut() {
                    if !child_table_mut.contains_key("parent") {
                        child_table_mut
                            .insert("parent".to_string(), LuaValue::String(aura.id.clone()));
                    }
                }

                prepared_children.insert(child_id.clone(), child_value);
            }
        }
    }

    // Second pass: set controlledChildren on subgroups (not the root)
    for (group_id, child_ids) in &children_by_parent {
        if group_id == &aura.id {
            continue;
        }
        if let Some(group_data) = prepared_children.get_mut(group_id) {
            set_controlled_children(group_data, child_ids);
        }
    }

    ChildrenHierarchy {
        children_by_parent,
        prepared_children,
    }
}

/// Set the `controlledChildren` field on a LuaValue table from a list of child IDs.
pub fn set_controlled_children(data: &mut LuaValue, child_ids: &[String]) {
    if let Some(table) = data.as_table_mut() {
        let controlled_children = LuaValue::Array(
            child_ids
                .iter()
                .map(|id| LuaValue::String(id.clone()))
                .collect(),
        );
        table.insert("controlledChildren".to_string(), controlled_children);
    }
}
