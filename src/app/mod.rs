//! Main GUI application for WeakAura Mass Import using iced.

mod actions;
mod state;
mod ui;

use std::collections::HashSet;
use std::path::PathBuf;

use arboard::Clipboard;
use iced::widget::{column, container, row};
use iced::{Element, Length, Task, Theme};

use crate::categories::UpdateCategory;
use crate::saved_variables::{
    AuraTreeNode, ConflictAction, ConflictDetectionResult, ImportResult, SavedVariablesInfo,
};
use crate::theme as app_theme;

pub use state::{ConflictResolutionUI, ImportUpdate, LoadingUpdate, ParsedAuraEntry, RemovalUpdate, ScanUpdate};

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
    /// Whether a background loading task is in progress
    pub(crate) is_loading: bool,
    /// Loading progress (0.0 to 1.0)
    pub(crate) loading_progress: f32,
    /// Loading progress message
    pub(crate) loading_message: String,
    /// Whether a background scanning task (loading SavedVariables) is in progress
    pub(crate) is_scanning: bool,
    /// Scanning progress message
    pub(crate) scanning_message: String,
    /// Whether a background removal task is in progress
    pub(crate) is_removing: bool,
    /// Removal progress message
    pub(crate) removal_message: String,
}

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
            is_loading: false,
            loading_progress: 0.0,
            loading_message: String::new(),
            is_scanning: false,
            scanning_message: String::new(),
            is_removing: false,
            removal_message: String::new(),
        }
    }
}

impl WeakAuraImporter {
    /// Create new application with initial state
    pub fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();

        // Auto-discover WoW installations
        let wow_paths = crate::saved_variables::SavedVariablesManager::find_wow_paths();
        if let Some(first_path) = wow_paths.first() {
            app.wow_path = first_path.to_string_lossy().to_string();
            app.scan_saved_variables_sync();
        }

        (app, Task::none())
    }

    /// Return the application theme
    pub fn theme(&self) -> Theme {
        app_theme::create_theme()
    }

    /// Update the application state based on messages
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Input handling
            Message::InputTextChanged(text) => {
                self.input_text = text;
                Task::none()
            }
            Message::WowPathChanged(path) => {
                self.wow_path = path;
                self.scan_saved_variables_sync();
                Task::none()
            }

            // File operations
            Message::LoadFromFile => self.load_from_file_async(),
            Message::LoadFromFolder => self.load_from_folder_async(),
            Message::BrowseWowPath => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|h| h.path().to_path_buf())
                },
                Message::WowPathSelected,
            ),
            Message::SelectSavedVariablesFile(path) => {
                self.selected_sv_path = Some(path);
                self.load_existing_auras_async()
            }
            Message::SelectSavedVariablesManually => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Lua files", &["lua"])
                        .pick_file()
                        .await
                        .map(|h| h.path().to_path_buf())
                },
                Message::ManualSvSelected,
            ),

            // File dialog results
            Message::FileSelected(path) => {
                if let Some(p) = path {
                    self.load_file_content_async(p)
                } else {
                    Task::none()
                }
            }
            Message::FolderSelected(path) => {
                if let Some(p) = path {
                    self.load_folder_content_async(p)
                } else {
                    Task::none()
                }
            }
            Message::WowPathSelected(path) => {
                if let Some(p) = path {
                    self.wow_path = p.to_string_lossy().to_string();
                    self.scan_saved_variables_sync();
                }
                Task::none()
            }
            Message::ManualSvSelected(path) => {
                if let Some(p) = path {
                    self.selected_sv_path = Some(p);
                    return self.load_existing_auras_async();
                }
                Task::none()
            }

            // Input actions
            Message::TogglePasteInput => {
                self.show_paste_input = !self.show_paste_input;
                Task::none()
            }
            Message::PasteFromClipboard => {
                self.paste_from_clipboard();
                Task::none()
            }
            Message::ParseInput => {
                self.parse_input();
                Task::none()
            }
            Message::ClearInput => {
                self.input_text.clear();
                self.parsed_auras.clear();
                self.show_paste_input = false;
                Task::none()
            }

            // View actions
            Message::ToggleDecodedView => {
                self.show_decoded_view = !self.show_decoded_view;
                Task::none()
            }
            Message::SelectAuraForPreview(idx) => {
                self.selected_aura_index = Some(idx);
                Task::none()
            }

            // Selection actions
            Message::ToggleAuraSelection(idx) => {
                if let Some(entry) = self.parsed_auras.get_mut(idx) {
                    entry.selected = !entry.selected;
                }
                Task::none()
            }
            Message::SelectAllAuras => {
                for entry in &mut self.parsed_auras {
                    if entry.validation.is_valid {
                        entry.selected = true;
                    }
                }
                Task::none()
            }
            Message::DeselectAllAuras => {
                for entry in &mut self.parsed_auras {
                    entry.selected = false;
                }
                Task::none()
            }
            Message::RemoveAuraFromList(idx) => {
                if idx < self.parsed_auras.len() {
                    self.parsed_auras.remove(idx);
                    match self.selected_aura_index {
                        Some(sel) if sel == idx => self.selected_aura_index = None,
                        Some(sel) if sel > idx => self.selected_aura_index = Some(sel - 1),
                        _ => {}
                    }
                }
                Task::none()
            }
            Message::RemoveSelectedFromList => {
                self.parsed_auras.retain(|e| !e.selected);
                self.selected_aura_index = None;
                Task::none()
            }

            // Import actions
            Message::ShowImportConfirm => {
                self.show_import_confirm = true;
                Task::none()
            }
            Message::HideImportConfirm => {
                self.show_import_confirm = false;
                Task::none()
            }
            Message::ConfirmImport => {
                self.show_import_confirm = false;
                self.import_auras_async()
            }

            // Conflict resolution
            Message::HideConflictDialog => {
                self.show_conflict_dialog = false;
                self.conflict_result = None;
                self.conflict_resolutions.clear();
                Task::none()
            }
            Message::SetConflictAction(idx, action) => {
                if let Some(res) = self.conflict_resolutions.get_mut(idx) {
                    res.action = action;
                    if action == ConflictAction::UpdateSelected {
                        res.categories = self.global_categories.clone();
                    }
                }
                Task::none()
            }
            Message::ToggleConflictExpanded(idx) => {
                if let Some(res) = self.conflict_resolutions.get_mut(idx) {
                    res.expanded = !res.expanded;
                }
                Task::none()
            }
            Message::ToggleGlobalCategory(category) => {
                if self.global_categories.contains(&category) {
                    self.global_categories.remove(&category);
                } else {
                    self.global_categories.insert(category);
                }
                // Update all resolutions that use UpdateSelected
                for res in &mut self.conflict_resolutions {
                    if res.action == ConflictAction::UpdateSelected {
                        res.categories = self.global_categories.clone();
                    }
                }
                Task::none()
            }
            Message::ToggleConflictCategory(idx, category) => {
                if let Some(res) = self.conflict_resolutions.get_mut(idx) {
                    if res.categories.contains(&category) {
                        res.categories.remove(&category);
                    } else {
                        res.categories.insert(category);
                    }
                }
                Task::none()
            }
            Message::SetAllConflictsAction(action) => {
                for res in &mut self.conflict_resolutions {
                    res.action = action;
                    if action == ConflictAction::UpdateSelected {
                        res.categories = self.global_categories.clone();
                    }
                }
                Task::none()
            }
            Message::ConfirmConflictResolutions => self.complete_import_with_resolutions_async(),

            // Removal actions
            Message::ToggleAuraForRemoval(id) => {
                if self.selected_for_removal.contains(&id) {
                    self.selected_for_removal.remove(&id);
                } else {
                    self.selected_for_removal.insert(id);
                }
                Task::none()
            }
            Message::SelectAllForRemoval => {
                fn collect_ids(node: &AuraTreeNode, set: &mut HashSet<String>) {
                    set.insert(node.id.clone());
                    for child in &node.children {
                        collect_ids(child, set);
                    }
                }
                for node in &self.existing_auras_tree {
                    collect_ids(node, &mut self.selected_for_removal);
                }
                Task::none()
            }
            Message::DeselectAllForRemoval => {
                self.selected_for_removal.clear();
                Task::none()
            }
            Message::ShowRemoveConfirm => {
                self.pending_removal_ids = self.selected_for_removal.iter().cloned().collect();
                self.show_remove_confirm = true;
                Task::none()
            }
            Message::HideRemoveConfirm => {
                self.show_remove_confirm = false;
                self.pending_removal_ids.clear();
                Task::none()
            }
            Message::ConfirmRemoval => {
                self.show_remove_confirm = false;
                self.remove_auras_async()
            }

            // Tree navigation
            Message::ToggleGroupExpanded(id) => {
                if self.expanded_groups.contains(&id) {
                    self.expanded_groups.remove(&id);
                } else {
                    self.expanded_groups.insert(id);
                }
                Task::none()
            }
            Message::ExpandAllGroups => {
                fn collect_groups(node: &AuraTreeNode, set: &mut HashSet<String>) {
                    if node.is_group {
                        set.insert(node.id.clone());
                        for child in &node.children {
                            collect_groups(child, set);
                        }
                    }
                }
                for node in &self.existing_auras_tree {
                    collect_groups(node, &mut self.expanded_groups);
                }
                Task::none()
            }
            Message::CollapseAllGroups => {
                self.expanded_groups.clear();
                Task::none()
            }

            // Async task results
            Message::LoadingUpdate(update) => {
                self.handle_loading_update(update);
                Task::none()
            }
            Message::ImportUpdate(update) => {
                self.handle_import_update(update);
                Task::none()
            }
            Message::ScanUpdate(update) => {
                self.handle_scan_update(update);
                Task::none()
            }
            Message::RemovalUpdate(update) => {
                self.handle_removal_update(update);
                Task::none()
            }
        }
    }

    /// Render the application view
    pub fn view(&self) -> Element<'_, Message> {
        // Main layout: menu bar at top, status bar at bottom, content in middle
        let menu_bar = self.render_menu_bar();
        let status_bar = self.render_status_bar();

        // Content area with sidebar and main panel
        let sidebar = self.render_sidebar();
        let main_content = self.render_main_content();

        // Optional decoded panel on right
        let content_row: Element<Message> = if self.show_decoded_view {
            let decoded_panel = self.render_decoded_panel();
            row![sidebar, main_content, decoded_panel]
                .height(Length::Fill)
                .into()
        } else {
            row![sidebar, main_content].height(Length::Fill).into()
        };

        // Stack the dialogs on top using overlay pattern
        let mut main_view: Element<Message> = column![menu_bar, content_row, status_bar].into();

        // Modal overlays
        if self.show_import_confirm {
            main_view = self.overlay_import_confirmation(main_view);
        }
        if self.show_conflict_dialog {
            main_view = self.overlay_conflict_dialog(main_view);
        }
        if self.show_remove_confirm {
            main_view = self.overlay_remove_confirmation(main_view);
        }

        container(main_view)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
