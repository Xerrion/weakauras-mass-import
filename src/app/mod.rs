//! Main GUI application for WeakAura Mass Import.

mod actions;
mod state;
mod ui;

use std::collections::HashSet;
use std::path::PathBuf;

use arboard::Clipboard;
use eframe::egui;
use tokio::sync::mpsc;

use crate::categories::UpdateCategory;
use crate::saved_variables::{
    AuraTreeNode, ConflictDetectionResult, ImportResult, SavedVariablesInfo,
};
use crate::theme;

use state::{ConflictResolutionUI, ImportUpdate, LoadingUpdate, ParsedAuraEntry};

/// Main application state
pub struct WeakAuraImporter {
    /// Input text area content
    pub(crate) input_text: String,
    /// Parsed auras from input
    pub(crate) parsed_auras: Vec<ParsedAuraEntry>,
    /// Selected SavedVariables file
    pub(crate) selected_sv_path: Option<PathBuf>,
    /// Discovered SavedVariables files
    pub(crate) discovered_sv_files: Vec<SavedVariablesInfo>,
    /// Status message
    pub(crate) status_message: String,
    /// Status is error
    pub(crate) status_is_error: bool,
    /// Import result
    pub(crate) last_import_result: Option<ImportResult>,
    /// Show decoded view
    pub(crate) show_decoded_view: bool,
    /// Selected aura index for preview
    pub(crate) selected_aura_index: Option<usize>,
    /// Show import confirmation dialog
    pub(crate) show_import_confirm: bool,
    /// WoW path for scanning
    pub(crate) wow_path: String,
    /// Clipboard handler
    pub(crate) clipboard: Option<Clipboard>,
    /// Show paste input area
    pub(crate) show_paste_input: bool,
    /// Loaded existing auras as tree
    pub(crate) existing_auras_tree: Vec<AuraTreeNode>,
    /// Total count of existing auras
    pub(crate) existing_auras_count: usize,
    /// Show conflict resolution dialog
    pub(crate) show_conflict_dialog: bool,
    /// Current conflict detection result
    pub(crate) conflict_result: Option<ConflictDetectionResult>,
    /// Resolutions for each conflict (parallel to conflict_result.conflicts)
    pub(crate) conflict_resolutions: Vec<ConflictResolutionUI>,
    /// Global categories to update (used as default for all conflicts)
    pub(crate) global_categories: HashSet<UpdateCategory>,
    /// Selected conflict index in the dialog
    pub(crate) selected_conflict_index: Option<usize>,
    /// Expanded groups in the existing auras tree
    pub(crate) expanded_groups: HashSet<String>,
    /// Import progress (0.0 to 1.0)
    pub(crate) import_progress: f32,
    /// Is import currently in progress
    pub(crate) is_importing: bool,
    /// Import progress message
    pub(crate) import_progress_message: String,
    /// Auras selected for removal in the sidebar
    pub(crate) selected_for_removal: HashSet<String>,
    /// Show removal confirmation dialog
    pub(crate) show_remove_confirm: bool,
    /// IDs pending removal (populated when confirm dialog opens)
    pub(crate) pending_removal_ids: Vec<String>,
    /// Tokio runtime for async operations
    pub(crate) runtime: tokio::runtime::Runtime,
    /// Whether a background loading task is in progress
    pub(crate) is_loading: bool,
    /// Loading progress (0.0 to 1.0)
    pub(crate) loading_progress: f32,
    /// Loading progress message
    pub(crate) loading_message: String,
    /// Receiver for loading updates from background tasks
    pub(crate) loading_receiver: Option<mpsc::Receiver<LoadingUpdate>>,
    /// Receiver for import updates from background tasks
    pub(crate) import_receiver: Option<mpsc::Receiver<ImportUpdate>>,
    /// Whether a background scanning task (loading SavedVariables) is in progress
    pub(crate) is_scanning: bool,
    /// Scanning progress message
    pub(crate) scanning_message: String,
    /// Receiver for scan updates from background tasks
    pub(crate) scan_receiver: Option<mpsc::Receiver<state::ScanUpdate>>,
}

impl Default for WeakAuraImporter {
    fn default() -> Self {
        Self {
            input_text: String::new(),
            parsed_auras: Vec::new(),
            selected_sv_path: None,
            discovered_sv_files: Vec::new(),
            status_message: String::from("Ready. Load WeakAura strings from file or folder."),
            status_is_error: false,
            last_import_result: None,
            show_decoded_view: false,
            selected_aura_index: None,
            show_import_confirm: false,
            wow_path: String::new(),
            clipboard: Clipboard::new().ok(),
            show_paste_input: false,
            existing_auras_tree: Vec::new(),
            existing_auras_count: 0,
            show_conflict_dialog: false,
            conflict_result: None,
            conflict_resolutions: Vec::new(),
            global_categories: UpdateCategory::defaults(),
            selected_conflict_index: None,
            expanded_groups: HashSet::new(),
            import_progress: 0.0,
            is_importing: false,
            import_progress_message: String::new(),
            selected_for_removal: HashSet::new(),
            show_remove_confirm: false,
            pending_removal_ids: Vec::new(),
            runtime: tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"),
            is_loading: false,
            loading_progress: 0.0,
            loading_message: String::new(),
            loading_receiver: None,
            import_receiver: None,
            is_scanning: false,
            scanning_message: String::new(),
            scan_receiver: None,
        }
    }
}

impl eframe::App for WeakAuraImporter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process async results from background tasks
        self.poll_loading();
        self.poll_importing();
        self.poll_scanning();

        // Request repaint while any background task is in progress
        if self.is_loading || self.is_importing || self.is_scanning {
            ctx.request_repaint();
        }

        theme::configure_theme(ctx);
        self.render_menu_bar(ctx);
        self.render_status_bar(ctx);
        self.render_sidebar(ctx);
        self.render_decoded_panel(ctx);
        self.render_main_content(ctx);
        self.render_import_confirmation(ctx);
        self.render_conflict_dialog(ctx);
        self.render_remove_confirmation(ctx);
    }
}
