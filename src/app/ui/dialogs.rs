//! Import confirmation and conflict resolution dialogs.

use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, space, text, Column,
};
use iced::{Alignment, Element, Length, Padding};

use crate::saved_variables::{ConflictAction, ImportConflict};
use crate::theme::{self, colors};

use super::super::{Message, WeakAuraImporter};

impl WeakAuraImporter {
    /// Overlay the import confirmation dialog on top of the main view
    pub(crate) fn overlay_import_confirmation<'a>(
        &'a self,
        underlay: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let count = self
            .parsed_auras
            .iter()
            .filter(|e| e.selected && e.validation.is_valid)
            .count();

        let target_text = if let Some(path) = &self.selected_sv_path {
            format!(
                "Target: {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            )
        } else {
            String::new()
        };

        let dialog_content = column![
            text(format!("Import {} aura(s) to SavedVariables?", count)).size(16),
            space::vertical().height(Length::Fixed(8.0)),
            text(target_text).color(colors::TEXT_MUTED),
            space::vertical().height(Length::Fixed(16.0)),
            row![
                button(text("Cancel"))
                    .style(theme::button_secondary)
                    .on_press(Message::HideImportConfirm),
                space::horizontal(),
                button(text("Confirm Import").color(colors::BG_DARKEST))
                    .style(theme::button_primary)
                    .on_press(Message::ConfirmImport),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(4)
        .padding(20)
        .max_width(400);

        let dialog_box = container(dialog_content)
            .style(theme::container_modal)
            .padding(8);

        let centered_dialog = container(dialog_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        let backdrop = container(centered_dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::container_modal_backdrop);

        // Stack the backdrop on top of the underlay
        iced::widget::stack![underlay, backdrop].into()
    }

    /// Overlay the conflict resolution dialog on top of the main view
    pub(crate) fn overlay_conflict_dialog<'a>(
        &'a self,
        underlay: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let (new_count, conflict_count, conflicts) = match &self.conflict_result {
            Some(cr) => (cr.new_auras.len(), cr.conflicts.len(), cr.conflicts.clone()),
            None => {
                return underlay;
            }
        };

        // Header info
        let header = row![
            text(format!("{} new aura(s) will be added", new_count)).color(colors::SUCCESS),
            text(" | ").color(colors::TEXT_MUTED),
            text(format!("{} aura(s) already exist", conflict_count)).color(colors::GOLD),
        ]
        .spacing(8);

        // Global category selection header
        let global_cat_header = text("Default Categories to Update:").size(14);

        // Category checkboxes (simplified grid - row of 4)
        // Build rows without borrowing a local Vec
        let categories_grid = self.build_category_grid();

        // Bulk action buttons
        let bulk_actions = row![
            button(text("Import All").size(12))
                .style(theme::button_secondary)
                .on_press(Message::SetAllConflictsAction(
                    ConflictAction::UpdateSelected
                )),
            button(text("Skip All").size(12))
                .style(theme::button_secondary)
                .on_press(Message::SetAllConflictsAction(ConflictAction::Skip)),
            button(text("Replace All").size(12))
                .style(theme::button_secondary)
                .on_press(Message::SetAllConflictsAction(ConflictAction::ReplaceAll)),
        ]
        .spacing(8);

        // Conflict list
        let conflict_list = self.render_conflict_list(&conflicts);

        let conflict_list_container =
            container(scrollable(conflict_list).height(Length::Fixed(250.0)))
                .style(theme::container_elevated)
                .padding(8);

        // Action buttons
        let action_buttons = row![
            button(text("Cancel"))
                .style(theme::button_secondary)
                .on_press(Message::HideConflictDialog),
            space::horizontal(),
            button(text("Import").color(colors::BG_DARKEST))
                .style(theme::button_primary)
                .on_press(Message::ConfirmConflictResolutions),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let dialog_content = column![
            text("Import Conflicts Detected")
                .size(18)
                .color(colors::GOLD),
            space::vertical().height(Length::Fixed(8.0)),
            header,
            space::vertical().height(Length::Fixed(12.0)),
            global_cat_header,
            categories_grid,
            space::vertical().height(Length::Fixed(12.0)),
            text("Conflicting Auras:").size(14),
            bulk_actions,
            space::vertical().height(Length::Fixed(8.0)),
            conflict_list_container,
            space::vertical().height(Length::Fixed(12.0)),
            action_buttons,
        ]
        .spacing(4)
        .padding(20)
        .max_width(700);

        let dialog_box = container(dialog_content)
            .style(theme::container_modal)
            .padding(8);

        let centered_dialog = container(dialog_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        let backdrop = container(centered_dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::container_modal_backdrop);

        iced::widget::stack![underlay, backdrop].into()
    }

    /// Build the category checkbox grid without borrowing issues
    fn build_category_grid(&self) -> Column<'_, Message> {
        use crate::categories::UpdateCategory;

        // Row 1: Name, Display, Trigger, Load
        let row1 = row![
            checkbox(self.global_categories.contains(&UpdateCategory::Name))
                .label("Name")
                .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Name))
                .text_size(12),
            checkbox(self.global_categories.contains(&UpdateCategory::Display))
                .label("Display")
                .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Display))
                .text_size(12),
            checkbox(self.global_categories.contains(&UpdateCategory::Trigger))
                .label("Trigger")
                .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Trigger))
                .text_size(12),
            checkbox(self.global_categories.contains(&UpdateCategory::Load))
                .label("Load")
                .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Load))
                .text_size(12),
        ]
        .spacing(12);

        // Row 2: Action, Animation, Conditions, Author Options
        let row2 = row![
            checkbox(self.global_categories.contains(&UpdateCategory::Action))
                .label("Actions")
                .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Action))
                .text_size(12),
            checkbox(self.global_categories.contains(&UpdateCategory::Animation))
                .label("Animations")
                .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Animation))
                .text_size(12),
            checkbox(self.global_categories.contains(&UpdateCategory::Conditions))
                .label("Conditions")
                .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Conditions))
                .text_size(12),
            checkbox(
                self.global_categories
                    .contains(&UpdateCategory::AuthorOptions)
            )
            .label("Author Options")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::AuthorOptions))
            .text_size(12),
        ]
        .spacing(12);

        // Row 3: Arrangement, Anchor, User Config, Metadata
        let row3 = row![
            checkbox(
                self.global_categories
                    .contains(&UpdateCategory::Arrangement)
            )
            .label("Arrangement")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Arrangement))
            .text_size(12),
            checkbox(self.global_categories.contains(&UpdateCategory::Anchor))
                .label("Anchor")
                .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Anchor))
                .text_size(12),
            checkbox(self.global_categories.contains(&UpdateCategory::UserConfig))
                .label("User Config")
                .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::UserConfig))
                .text_size(12),
            checkbox(self.global_categories.contains(&UpdateCategory::Metadata))
                .label("Metadata")
                .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Metadata))
                .text_size(12),
        ]
        .spacing(12);

        Column::new().push(row1).push(row2).push(row3).spacing(4)
    }

    fn render_conflict_list(&self, conflicts: &[ImportConflict]) -> Column<'_, Message> {
        let mut list_col = Column::new().spacing(8);

        for (idx, conflict) in conflicts.iter().enumerate() {
            let resolution = &self.conflict_resolutions[idx];

            // Action dropdown using pick_list
            let action_options = vec![
                ConflictAction::Skip,
                ConflictAction::ReplaceAll,
                ConflictAction::UpdateSelected,
            ];

            let action_picker = pick_list(action_options, Some(resolution.action), move |action| {
                Message::SetConflictAction(idx, action)
            })
            .text_size(12)
            .width(Length::Fixed(90.0));

            // Aura name with color based on action
            let name_color = match resolution.action {
                ConflictAction::Skip => colors::TEXT_MUTED,
                _ => colors::TEXT_PRIMARY,
            };

            let aura_id = conflict.aura_id.clone();
            let mut item_row = row![action_picker, text(aura_id).color(name_color)]
                .spacing(8)
                .align_y(Alignment::Center);

            // Group indicator
            if conflict.is_group {
                item_row = item_row.push(
                    text(format!("[Group: {} children]", conflict.child_count))
                        .color(colors::TEXT_MUTED)
                        .size(12),
                );
            }

            // Changed categories indicator
            if !conflict.changed_categories.is_empty() {
                let changed_names: Vec<&str> = conflict
                    .changed_categories
                    .iter()
                    .map(|c| c.display_name())
                    .collect();
                item_row = item_row.push(
                    text(format!("Changes: {}", changed_names.join(", ")))
                        .color(colors::TEXT_MUTED)
                        .size(12),
                );
            }

            // Expand button for per-aura category selection
            if resolution.action == ConflictAction::UpdateSelected {
                let expand_text = if resolution.expanded { "v" } else { ">" };
                item_row = item_row.push(
                    button(text(expand_text).size(12))
                        .style(theme::button_frameless)
                        .on_press(Message::ToggleConflictExpanded(idx)),
                );
            }

            list_col = list_col.push(item_row);

            // Expanded category selection for this specific aura
            if resolution.expanded && resolution.action == ConflictAction::UpdateSelected {
                list_col =
                    list_col.push(self.build_conflict_category_grid(idx, resolution, conflict));
            }

            // Separator
            list_col = list_col.push(
                container(text(""))
                    .height(Length::Fixed(1.0))
                    .width(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(colors::BORDER.into()),
                        ..Default::default()
                    }),
            );
        }

        list_col
    }

    /// Build the category grid for a specific conflict's expanded view
    fn build_conflict_category_grid<'a>(
        &'a self,
        idx: usize,
        resolution: &'a crate::app::ConflictResolutionUI,
        _conflict: &ImportConflict,
    ) -> Column<'a, Message> {
        use crate::categories::UpdateCategory;

        let indent = space::horizontal().width(Length::Fixed(100.0));

        // Row 1: Name, Display, Trigger, Load
        let row1 = row![
            indent,
            checkbox(resolution.categories.contains(&UpdateCategory::Name))
                .label("Name")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Name))
                .text_size(11),
            checkbox(resolution.categories.contains(&UpdateCategory::Display))
                .label("Display")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Display))
                .text_size(11),
            checkbox(resolution.categories.contains(&UpdateCategory::Trigger))
                .label("Trigger")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Trigger))
                .text_size(11),
            checkbox(resolution.categories.contains(&UpdateCategory::Load))
                .label("Load")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Load))
                .text_size(11),
        ]
        .spacing(8);

        // Row 2: Action, Animation, Conditions, Author Options
        let row2 = row![
            space::horizontal().width(Length::Fixed(100.0)),
            checkbox(resolution.categories.contains(&UpdateCategory::Action))
                .label("Actions")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Action))
                .text_size(11),
            checkbox(resolution.categories.contains(&UpdateCategory::Animation))
                .label("Animations")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Animation))
                .text_size(11),
            checkbox(resolution.categories.contains(&UpdateCategory::Conditions))
                .label("Conditions")
                .on_toggle(move |_| Message::ToggleConflictCategory(
                    idx,
                    UpdateCategory::Conditions
                ))
                .text_size(11),
            checkbox(
                resolution
                    .categories
                    .contains(&UpdateCategory::AuthorOptions)
            )
            .label("Author Options")
            .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::AuthorOptions))
            .text_size(11),
        ]
        .spacing(8);

        // Row 3: Arrangement, Anchor, User Config, Metadata
        let row3 = row![
            space::horizontal().width(Length::Fixed(100.0)),
            checkbox(resolution.categories.contains(&UpdateCategory::Arrangement))
                .label("Arrangement")
                .on_toggle(move |_| Message::ToggleConflictCategory(
                    idx,
                    UpdateCategory::Arrangement
                ))
                .text_size(11),
            checkbox(resolution.categories.contains(&UpdateCategory::Anchor))
                .label("Anchor")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Anchor))
                .text_size(11),
            checkbox(resolution.categories.contains(&UpdateCategory::UserConfig))
                .label("User Config")
                .on_toggle(move |_| Message::ToggleConflictCategory(
                    idx,
                    UpdateCategory::UserConfig
                ))
                .text_size(11),
            checkbox(resolution.categories.contains(&UpdateCategory::Metadata))
                .label("Metadata")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Metadata))
                .text_size(11),
        ]
        .spacing(8);

        Column::new()
            .push(row1)
            .push(row2)
            .push(row3)
            .spacing(2)
            .padding(Padding::default().bottom(8.0))
    }

    /// Overlay the remove confirmation dialog on top of the main view
    pub(crate) fn overlay_remove_confirmation<'a>(
        &'a self,
        underlay: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let count = self.pending_removal_ids.len();

        // List of IDs to be removed
        let mut id_list = Column::new().spacing(4);
        for id in &self.pending_removal_ids {
            id_list = id_list.push(text(id).color(colors::TEXT_SECONDARY).size(13));
        }

        let id_list_container = container(scrollable(id_list).height(Length::Fixed(150.0)))
            .style(theme::container_elevated)
            .padding(8)
            .width(Length::Fill);

        let dialog_content = column![
            text(format!("Remove {} aura(s) from SavedVariables?", count)).size(16),
            space::vertical().height(Length::Fixed(4.0)),
            text("Groups will have all their children removed recursively.")
                .color(colors::TEXT_MUTED)
                .size(12),
            space::vertical().height(Length::Fixed(8.0)),
            id_list_container,
            space::vertical().height(Length::Fixed(12.0)),
            row![
                button(text("Cancel"))
                    .style(theme::button_secondary)
                    .on_press(Message::HideRemoveConfirm),
                space::horizontal(),
                button(text("Remove").color(colors::BG_DARKEST))
                    .style(theme::button_danger)
                    .on_press(Message::ConfirmRemoval),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(4)
        .padding(20)
        .max_width(450);

        let dialog_box = container(dialog_content)
            .style(theme::container_modal)
            .padding(8);

        let centered_dialog = container(dialog_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        let backdrop = container(centered_dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::container_modal_backdrop);

        iced::widget::stack![underlay, backdrop].into()
    }
}
