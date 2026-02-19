use std::fs;
use std::path::PathBuf;

use weakauras_mass_import::decoder::{LuaValue, WeakAuraDecoder};
use weakauras_mass_import::saved_variables::{
    ConflictAction, ConflictResolution, SavedVariablesManager,
};

fn main() {
    let repo_root = std::env::current_dir().expect("Failed to get current dir");
    let vars_path = repo_root.join("WeakAuras_vars.lua");
    let v1_path = repo_root.join("wlock v1.txt");
    let v2_path = repo_root.join("wlock v2.txt");

    let v1 = read_file(&v1_path);
    let v2 = read_file(&v2_path);

    let mut manager = SavedVariablesManager::new(vars_path.clone());
    manager.load().expect("Failed to load WeakAuras_vars.lua");

    println!("Loaded SavedVariables from {}", vars_path.display());
    println!("Displays before import: {}", manager.displays.len());

    let aura_v1 = WeakAuraDecoder::decode(v1.trim()).expect("Failed to decode v1");
    let aura_v2 = WeakAuraDecoder::decode(v2.trim()).expect("Failed to decode v2");

    println!("\n=== Import v1 ===");
    apply_replace_all(&mut manager, &aura_v1);
    println!("Displays after v1: {}", manager.displays.len());
    summarize_root(&manager, &aura_v1.id);

    println!("\n=== Import v2 ===");
    apply_replace_all(&mut manager, &aura_v2);
    println!("Displays after v2: {}", manager.displays.len());
    summarize_root(&manager, &aura_v2.id);

    let output_path = vars_path.with_extension("lua.updated");
    manager
        .save_as(&output_path)
        .expect("Failed to save updated file");
    println!("\nUpdated file written to {}", output_path.display());
}

fn read_file(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path.display()))
}

fn apply_replace_all(
    manager: &mut SavedVariablesManager,
    aura: &weakauras_mass_import::decoder::WeakAura,
) {
    let conflicts = manager.detect_conflicts(std::slice::from_ref(aura));
    if conflicts.conflicts.is_empty() && conflicts.new_auras.is_empty() {
        println!("No changes detected for {}", aura.id);
        return;
    }

    if !conflicts.new_auras.is_empty() {
        let new_ids: Vec<String> = conflicts
            .new_auras
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        println!("New auras: {}", new_ids.len());
        println!("New IDs: {:?}", new_ids);
    }

    if !conflicts.new_auras.is_empty() {
        let new_ids: Vec<String> = conflicts
            .new_auras
            .iter()
            .map(|(id, _)| id.clone())
            .collect();
        println!("New auras: {}", new_ids.len());
        println!("New IDs: {:?}", new_ids);
    }

    let mut resolutions = Vec::new();
    for conflict in &conflicts.conflicts {
        resolutions.push(ConflictResolution {
            aura_id: conflict.aura_id.clone(),
            action: ConflictAction::ReplaceAll,
            categories_to_update: Default::default(),
        });
    }

    let result = manager.apply_resolutions(&conflicts, &resolutions);
    println!("Applied: {}", result.summary());
}

fn summarize_root(manager: &SavedVariablesManager, root_id: &str) {
    let Some(root) = manager.displays.get(root_id) else {
        println!("Root {} not found", root_id);
        return;
    };

    let Some(table) = root.as_table() else {
        println!("Root {} is not a table", root_id);
        return;
    };

    let controlled = table
        .get("controlledChildren")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let children: Vec<String> = controlled
        .iter()
        .filter_map(|value| match value {
            LuaValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .collect();

    println!(
        "Root {} has {} controlled children",
        root_id,
        children.len()
    );
    println!("Controlled children: {:?}", children);
}
