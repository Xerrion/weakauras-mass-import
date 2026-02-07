//! Business logic actions for WeakAuraImporter, organized by concern.
//!
//! This module handles all async operations and state updates for the GUI:
//! - `handlers`: Message update handlers for async task results
//! - `import`: Import auras to SavedVariables (with conflict resolution)
//! - `loading`: Load auras from files, folders, clipboard, and text input
//! - `removal`: Remove auras and scan SavedVariables

mod handlers;
mod import;
mod loading;
mod removal;

use std::collections::HashSet;

use iced_toasts::{toast, ToastContainer, ToastLevel};

use crate::decoder::{ValidationResult, WeakAuraDecoder};

use super::state::ParsedAuraEntry;
use super::Message;

// Re-export all impl blocks from submodules for the parent module
// The wildcard re-exports make all `impl WeakAuraImporter` methods available
#[allow(unused_imports)]
pub(crate) use handlers::*;
#[allow(unused_imports)]
pub(crate) use import::*;
#[allow(unused_imports)]
pub(crate) use loading::*;
#[allow(unused_imports)]
pub(crate) use removal::*;

/// Collect the set of aura IDs already present in the parsed auras list.
pub(crate) fn collect_existing_ids(parsed_auras: &[ParsedAuraEntry]) -> HashSet<String> {
    parsed_auras
        .iter()
        .filter_map(|e| e.validation.aura_id.as_deref())
        .map(|id| id.to_string())
        .collect()
}

/// Decode auras from content, filtering out duplicates already in `existing_ids`.
/// Returns `(entries, added, duplicates, errors)` where errors is a list of error messages.
/// Invalid entries are NOT added to the entries list.
pub(crate) fn decode_auras_filtered(
    content: &str,
    existing_ids: &HashSet<String>,
) -> (Vec<ParsedAuraEntry>, usize, usize, Vec<String>) {
    let results = WeakAuraDecoder::decode_multiple(content);
    let mut entries = Vec::new();
    let mut added = 0;
    let mut duplicates = 0;
    let mut errors = Vec::new();

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
                    aura_id: Some(aura.id.clone()),
                    is_group: aura.is_group,
                    child_count: aura.children.len(),
                    error: None,
                };
                entries.push(ParsedAuraEntry {
                    validation,
                    aura: Some(aura),
                    selected: false,
                });
            }
            Err(e) => {
                errors.push(e.to_string());
            }
        }
    }

    (entries, added, duplicates, errors)
}

/// Show toast notifications for decode results.
/// Consolidates the repeated pattern of notifying users about added/duplicates/errors.
pub(crate) fn notify_decode_results(
    toasts: &mut ToastContainer<'static, Message>,
    added: usize,
    duplicates: usize,
    errors: &[String],
    context: &str, // e.g., "added", "loaded"
) {
    // Show error toasts for invalid strings
    for error in errors {
        toasts.push(
            toast(error)
                .title("Invalid WeakAura")
                .level(ToastLevel::Error),
        );
    }

    if added == 0 && duplicates == 0 && errors.is_empty() {
        toasts.push(
            toast(&format!("No WeakAura strings found in {}", context)).level(ToastLevel::Warning),
        );
    } else if added > 0 {
        let mut msg = format!("{} aura(s) {}", added, context);
        if duplicates > 0 {
            msg.push_str(&format!(", {} duplicate(s) skipped", duplicates));
        }
        toasts.push(toast(&msg).title("Success").level(ToastLevel::Success));
    } else if duplicates > 0 && errors.is_empty() {
        toasts.push(toast(&format!("{} duplicate(s) skipped", duplicates)).level(ToastLevel::Info));
    }
}
