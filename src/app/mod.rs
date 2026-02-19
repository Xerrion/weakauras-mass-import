//! Main GUI application for WeakAura Mass Import using iced.

mod actions;
mod message;
mod state;
mod ui;

pub use message::Message;

use std::collections::HashSet;

use arboard::Clipboard;
use iced::widget::{column, container, row, text};
use iced::{Element, Length, Task, Theme};
use iced_toasts::{toast_container, ToastContainer};

use crate::saved_variables::{AuraTreeNode, ConflictAction};
use crate::theme as app_theme;

pub use state::{ConflictResolutionUI, ParsedAuraEntry};
use state::{
    ConflictState, RemovalState, SavedVariablesState, SidebarState, StatusState, TaskProgress,
    UiVisibility,
};

/// Main application state
pub struct WeakAuraImporter {
    /// Input text area content
    pub(crate) input_text: String,
    /// Parsed auras from input
    pub(crate) parsed_auras: Vec<ParsedAuraEntry>,
    /// Selected aura index for preview
    pub(crate) selected_aura_index: Option<usize>,
    /// Clipboard handler
    pub(crate) clipboard: Option<Clipboard>,
    /// Toast notification container
    pub(crate) toasts: ToastContainer<'static, Message>,
    /// UI visibility state
    pub(crate) ui: UiVisibility,
    /// Sidebar state
    pub(crate) sidebar: SidebarState,
    /// Task progress state
    pub(crate) tasks: TaskProgress,
    /// Conflict resolution state
    pub(crate) conflicts: ConflictState,
    /// Aura removal state
    pub(crate) removal: RemovalState,
    /// SavedVariables state
    pub(crate) saved_vars: SavedVariablesState,
    /// Status bar state
    pub(crate) status: StatusState,
}

impl Default for WeakAuraImporter {
    fn default() -> Self {
        Self {
            input_text: String::new(),
            parsed_auras: Vec::new(),
            selected_aura_index: None,
            clipboard: Clipboard::new().ok(),
            toasts: toast_container(Message::DismissToast),
            ui: UiVisibility {
                show_setup_wizard: true,
                ..UiVisibility::default()
            },
            sidebar: SidebarState::default(),
            tasks: TaskProgress::default(),
            conflicts: ConflictState::default(),
            removal: RemovalState::default(),
            saved_vars: SavedVariablesState::default(),
            status: StatusState::default(),
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
            app.saved_vars.wow_path = first_path.to_string_lossy().to_string();
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
                self.saved_vars.wow_path = path;
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
                self.saved_vars.selected_path = Some(path);
                self.removal.selected_ids.clear();
                self.removal.pending_ids.clear();
                // Don't load yet - wait for Continue button
                Task::none()
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
                    self.saved_vars.wow_path = p.to_string_lossy().to_string();
                    self.scan_saved_variables_sync();
                }
                Task::none()
            }
            Message::ManualSvSelected(path) => {
                if let Some(p) = path {
                    self.saved_vars.selected_path = Some(p);
                    self.removal.selected_ids.clear();
                    self.removal.pending_ids.clear();
                    return self.load_existing_auras_async();
                }
                Task::none()
            }
            Message::ShowSetupWizard => {
                self.ui.show_setup_wizard = true;
                Task::none()
            }
            Message::HideSetupWizard => {
                if self.saved_vars.selected_path.is_some() {
                    self.ui.show_setup_wizard = false;
                    // Load the SavedVariables when continuing
                    return self.load_existing_auras_async();
                }
                Task::none()
            }

            // Input actions
            Message::TogglePasteInput => {
                self.ui.show_paste_input = !self.ui.show_paste_input;
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
                self.ui.show_paste_input = false;
                Task::none()
            }

            // View actions
            Message::ToggleDecodedView => {
                self.ui.show_decoded_view = !self.ui.show_decoded_view;
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
                self.ui.show_import_confirm = true;
                Task::none()
            }
            Message::HideImportConfirm => {
                self.ui.show_import_confirm = false;
                Task::none()
            }
            Message::ConfirmImport => {
                self.ui.show_import_confirm = false;
                self.import_auras_async()
            }

            // Conflict resolution
            Message::HideConflictDialog => {
                self.ui.show_conflict_dialog = false;
                self.conflicts.result = None;
                self.conflicts.resolutions.clear();
                Task::none()
            }
            Message::SetConflictAction(idx, action) => {
                if let Some(res) = self.conflicts.resolutions.get_mut(idx) {
                    res.action = action;
                    if action == ConflictAction::UpdateSelected {
                        res.categories = self.conflicts.global_categories.clone();
                    }
                }
                Task::none()
            }
            Message::ToggleConflictExpanded(idx) => {
                if let Some(res) = self.conflicts.resolutions.get_mut(idx) {
                    res.expanded = !res.expanded;
                }
                Task::none()
            }
            Message::ToggleGlobalCategory(category) => {
                if self.conflicts.global_categories.contains(&category) {
                    self.conflicts.global_categories.remove(&category);
                } else {
                    self.conflicts.global_categories.insert(category);
                }
                // Update all resolutions that use UpdateSelected
                for res in &mut self.conflicts.resolutions {
                    if res.action == ConflictAction::UpdateSelected {
                        res.categories = self.conflicts.global_categories.clone();
                    }
                }
                Task::none()
            }
            Message::ToggleConflictCategory(idx, category) => {
                if let Some(res) = self.conflicts.resolutions.get_mut(idx) {
                    if res.categories.contains(&category) {
                        res.categories.remove(&category);
                    } else {
                        res.categories.insert(category);
                    }
                }
                Task::none()
            }
            Message::SetAllConflictsAction(action) => {
                for res in &mut self.conflicts.resolutions {
                    res.action = action;
                    if action == ConflictAction::UpdateSelected {
                        res.categories = self.conflicts.global_categories.clone();
                    }
                }
                Task::none()
            }
            Message::ConfirmConflictResolutions => self.complete_import_with_resolutions_async(),

            // Removal actions
            Message::ToggleAuraForRemoval(id) => {
                if self.removal.selected_ids.contains(&id) {
                    self.removal.selected_ids.remove(&id);
                } else {
                    self.removal.selected_ids.insert(id);
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
                for node in &self.saved_vars.auras_tree {
                    collect_ids(node, &mut self.removal.selected_ids);
                }
                Task::none()
            }
            Message::DeselectAllForRemoval => {
                self.removal.selected_ids.clear();
                Task::none()
            }
            Message::ShowRemoveConfirm => {
                self.removal.pending_ids = self.removal.selected_ids.iter().cloned().collect();
                self.ui.show_remove_confirm = true;
                Task::none()
            }
            Message::HideRemoveConfirm => {
                self.ui.show_remove_confirm = false;
                self.removal.pending_ids.clear();
                Task::none()
            }
            Message::ConfirmRemoval => {
                self.ui.show_remove_confirm = false;
                self.remove_auras_async()
            }

            // Tree navigation
            Message::ToggleGroupExpanded(id) => {
                if self.sidebar.expanded_groups.contains(&id) {
                    self.sidebar.expanded_groups.remove(&id);
                } else {
                    self.sidebar.expanded_groups.insert(id);
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
                for node in &self.saved_vars.auras_tree {
                    collect_groups(node, &mut self.sidebar.expanded_groups);
                }
                Task::none()
            }
            Message::CollapseAllGroups => {
                self.sidebar.expanded_groups.clear();
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

            // Sidebar resize
            Message::StartSidebarResize => {
                self.sidebar.is_resizing = true;
                Task::none()
            }
            Message::SidebarResize(x) => {
                if self.sidebar.is_resizing {
                    // Clamp sidebar width between min and max
                    const MIN_WIDTH: f32 = 200.0;
                    const MAX_WIDTH: f32 = 600.0;
                    self.sidebar.width = x.clamp(MIN_WIDTH, MAX_WIDTH);
                }
                Task::none()
            }
            Message::EndSidebarResize => {
                self.sidebar.is_resizing = false;
                Task::none()
            }
            Message::HoverResizeEdge => {
                self.sidebar.is_hovering_resize = true;
                Task::none()
            }
            Message::UnhoverResizeEdge => {
                self.sidebar.is_hovering_resize = false;
                Task::none()
            }

            // Toast notifications
            Message::DismissToast(id) => {
                self.toasts.dismiss(id);
                Task::none()
            }
        }
    }

    /// Render the application view
    pub fn view(&self) -> Element<'_, Message> {
        use crate::theme::{colors, spacing, typography};
        use iced::mouse::Interaction;
        use iced::widget::{mouse_area, space};

        // Header with app title and menu
        let header = container(
            row![
                text("WeakAuras Mass Importer")
                    .size(typography::TITLE)
                    .color(colors::GOLD),
                space::horizontal(),
                self.render_menu_buttons()
            ]
            .spacing(spacing::MD)
            .padding(spacing::MD)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .style(app_theme::container_toolbar);

        let sidebar = self.render_sidebar();
        let main_content = self.render_main_content();

        // Resize edge - thin strip on the right of sidebar, visible on hover
        let resize_color = if self.sidebar.is_resizing {
            colors::GOLD
        } else if self.sidebar.is_hovering_resize {
            colors::GOLD_MUTED
        } else {
            iced::Color::TRANSPARENT
        };

        let resize_edge = mouse_area(
            container(space::horizontal())
                .width(Length::Fixed(4.0))
                .height(Length::Fill)
                .style(move |_theme| container::Style {
                    background: Some(resize_color.into()),
                    ..Default::default()
                }),
        )
        .interaction(Interaction::ResizingHorizontally)
        .on_press(Message::StartSidebarResize)
        .on_release(Message::EndSidebarResize)
        .on_enter(Message::HoverResizeEdge)
        .on_exit(Message::UnhoverResizeEdge);

        // When resizing, we need to track mouse movement globally
        let content_row: Element<Message> = if self.ui.show_decoded_view {
            let decoded_panel = self.render_decoded_panel();
            row![sidebar, resize_edge, main_content, decoded_panel]
                .spacing(0)
                .height(Length::Fill)
                .into()
        } else {
            row![sidebar, resize_edge, main_content]
                .spacing(0)
                .height(Length::Fill)
                .into()
        };

        // Wrap content in mouse_area to track resize dragging
        let content_with_resize: Element<Message> = if self.sidebar.is_resizing {
            mouse_area(content_row)
                .on_move(|point| Message::SidebarResize(point.x))
                .on_release(Message::EndSidebarResize)
                .into()
        } else {
            content_row
        };

        let content_container = container(content_with_resize)
            .padding(spacing::MD)
            .width(Length::Fill)
            .height(Length::Fill);

        // Status bar with progress
        let status_bar = self.render_status_bar();

        // Stack the dialogs on top using overlay pattern
        let mut main_view: Element<Message> = column![header, content_container, status_bar].into();

        // Modal overlays
        if self.ui.show_import_confirm {
            main_view = self.overlay_import_confirmation(main_view);
        }
        if self.ui.show_conflict_dialog {
            main_view = self.overlay_conflict_dialog(main_view);
        }
        if self.ui.show_remove_confirm {
            main_view = self.overlay_remove_confirmation(main_view);
        }
        if self.ui.show_setup_wizard || self.saved_vars.selected_path.is_none() {
            main_view = self.overlay_setup_wizard(main_view);
        }

        let final_view = container(main_view)
            .width(Length::Fill)
            .height(Length::Fill);

        // Wrap with toast notifications overlay
        self.toasts.view(final_view)
    }
}
