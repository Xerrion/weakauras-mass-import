//! Import confirmation and conflict resolution dialogs.

use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, space, text, text_input,
    Column,
};
use iced::{Alignment, Element, Length, Padding};

use crate::saved_variables::{ConflictAction, ImportConflict};
use crate::theme::{self, colors, spacing, typography};

use super::super::{Message, WeakAuraImporter};

impl WeakAuraImporter {
    /// Overlay the setup wizard for selecting SavedVariables
    pub(crate) fn overlay_setup_wizard<'a>(
        &'a self,
        underlay: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let mut content = Column::new().spacing(spacing::SM);

        content = content.push(
            text("Game Location")
                .size(typography::HEADING)
                .color(colors::GOLD),
        );

        let wow_path_input = text_input("WoW Path...", &self.saved_vars.wow_path)
            .on_input(Message::WowPathChanged)
            .style(theme::text_input_style);

        let browse_btn = button(text("...").size(typography::BODY))
            .style(theme::button_secondary)
            .on_press(Message::BrowseWowPath);

        content = content.push(
            column![
                text("WoW Path:")
                    .size(typography::BODY)
                    .color(colors::TEXT_SECONDARY),
                row![wow_path_input.width(Length::Fill), browse_btn].spacing(spacing::SM),
            ]
            .spacing(spacing::XS),
        );

        content = content.push(
            text("Discovered SavedVariables:")
                .size(typography::BODY)
                .color(colors::TEXT_PRIMARY),
        );

        let files_list = if self.saved_vars.discovered_files.is_empty() {
            column![text("No SavedVariables found")
                .size(typography::BODY)
                .color(colors::TEXT_MUTED)]
        } else {
            let mut files_col = Column::new().spacing(spacing::XS);
            for sv_info in &self.saved_vars.discovered_files {
                let is_selected = self.saved_vars.selected_path.as_ref() == Some(&sv_info.path);
                let label_text = format!("{} ({})", sv_info.account, sv_info.pretty_flavor());
                let path_clone = sv_info.path.clone();

                let file_btn = button(row![text(label_text).size(typography::BODY).color(
                    if is_selected {
                        colors::BG_VOID
                    } else {
                        colors::TEXT_SECONDARY
                    }
                )])
                .width(Length::Fill)
                .style(if is_selected {
                    theme::button_primary
                } else {
                    theme::button_frameless
                })
                .on_press(Message::SelectSavedVariablesFile(path_clone));

                files_col = files_col.push(file_btn);
            }
            files_col
        };

        let files_container = container(
            scrollable(files_list)
                .height(Length::Fixed(180.0))
                .style(theme::scrollable_style),
        )
        .style(theme::container_inset)
        .padding(spacing::SM);

        content = content.push(files_container);

        content = content.push(
            button(text("Select file manually...").size(typography::BODY))
                .style(theme::button_secondary)
                .width(Length::Fill)
                .on_press(Message::SelectSavedVariablesManually),
        );

        if self.tasks.is_scanning {
            content = content.push(
                row![
                    text("Loading...")
                        .size(typography::BODY)
                        .color(colors::TEXT_SECONDARY),
                    text(&self.tasks.scanning_message)
                        .size(typography::CAPTION)
                        .color(colors::TEXT_MUTED),
                ]
                .spacing(spacing::SM),
            );
        }

        let can_continue = self.saved_vars.selected_path.is_some();

        let cancel_btn = if can_continue {
            button(text("Cancel").size(typography::BODY))
                .style(theme::button_secondary)
                .on_press(Message::HideSetupWizard)
        } else {
            button(text("Cancel").size(typography::BODY)).style(theme::button_secondary)
        };

        let continue_btn = if can_continue {
            button(
                text("Continue")
                    .size(typography::BODY)
                    .color(colors::BG_VOID),
            )
            .style(theme::button_primary)
            .on_press(Message::HideSetupWizard)
        } else {
            button(text("Continue").size(typography::BODY)).style(theme::button_secondary)
        };

        let actions_row = row![cancel_btn, space::horizontal(), continue_btn]
            .spacing(spacing::SM)
            .align_y(Alignment::Center);

        let dialog_content = column![content, space::vertical().height(spacing::MD), actions_row]
            .spacing(spacing::SM)
            .padding(spacing::XL)
            .max_width(520);

        let dialog_box = container(dialog_content)
            .style(theme::container_modal)
            .padding(spacing::SM)
            .width(Length::Fixed(520.0));

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

        let target_text = if let Some(path) = &self.saved_vars.selected_path {
            format!(
                "Target: {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            )
        } else {
            String::new()
        };

        let dialog_content = column![
            text(format!("Import {} aura(s)?", count)).size(typography::HEADING),
            space::vertical().height(Length::Fixed(spacing::SM)),
            text(target_text)
                .size(typography::BODY)
                .color(colors::TEXT_MUTED),
            space::vertical().height(Length::Fixed(spacing::LG)),
            row![
                button(text("Cancel").size(typography::BODY))
                    .style(theme::button_secondary)
                    .on_press(Message::HideImportConfirm),
                space::horizontal(),
                button(
                    text("Confirm Import")
                        .size(typography::BODY)
                        .color(colors::BG_VOID)
                )
                .style(theme::button_primary)
                .on_press(Message::ConfirmImport),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::XS)
        .padding(spacing::XL)
        .max_width(400);

        let dialog_box = container(dialog_content)
            .style(theme::container_modal)
            .padding(spacing::SM)
            .width(Length::Fixed(400.0));

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
        let (new_count, conflict_count, conflicts) = match &self.conflicts.result {
            Some(cr) => (cr.new_auras.len(), cr.conflicts.len(), cr.conflicts.clone()),
            None => {
                return underlay;
            }
        };

        // Header info
        let header = row![
            text(format!("{} new aura(s) will be added", new_count))
                .size(typography::BODY)
                .color(colors::SUCCESS),
            text(" | ").size(typography::BODY).color(colors::TEXT_MUTED),
            text(format!("{} aura(s) already exist", conflict_count))
                .size(typography::BODY)
                .color(colors::GOLD),
        ]
        .spacing(spacing::SM);

        // Global category selection header
        let global_cat_header = text("Default categories to update:").size(typography::BODY);

        // Category checkboxes (simplified grid - row of 4)
        // Build rows without borrowing a local Vec
        let categories_grid = self.build_category_grid();

        // Bulk action buttons
        let bulk_actions = row![
            button(text("Import All").size(typography::CAPTION))
                .style(theme::button_secondary)
                .on_press(Message::SetAllConflictsAction(
                    ConflictAction::UpdateSelected
                )),
            button(text("Skip All").size(typography::CAPTION))
                .style(theme::button_secondary)
                .on_press(Message::SetAllConflictsAction(ConflictAction::Skip)),
            button(text("Replace All").size(typography::CAPTION))
                .style(theme::button_secondary)
                .on_press(Message::SetAllConflictsAction(ConflictAction::ReplaceAll)),
        ]
        .spacing(spacing::SM);

        // Conflict list
        let conflict_list = self.render_conflict_list(&conflicts);

        let conflict_list_container = container(
            scrollable(conflict_list)
                .height(Length::Fixed(250.0))
                .style(theme::scrollable_style),
        )
        .style(theme::container_elevated)
        .padding(spacing::SM);

        // Action buttons
        let action_buttons = row![
            button(text("Cancel").size(typography::BODY))
                .style(theme::button_secondary)
                .on_press(Message::HideConflictDialog),
            space::horizontal(),
            button(text("Import").size(typography::BODY).color(colors::BG_VOID))
                .style(theme::button_primary)
                .on_press(Message::ConfirmConflictResolutions),
        ]
        .spacing(spacing::SM)
        .align_y(Alignment::Center);

        let dialog_content = column![
            text("Conflicts detected")
                .size(typography::TITLE)
                .color(colors::GOLD),
            space::vertical().height(Length::Fixed(spacing::SM)),
            header,
            space::vertical().height(Length::Fixed(spacing::MD)),
            global_cat_header,
            categories_grid,
            space::vertical().height(Length::Fixed(spacing::MD)),
            text("Conflicts:").size(typography::BODY),
            bulk_actions,
            space::vertical().height(Length::Fixed(spacing::SM)),
            conflict_list_container,
            space::vertical().height(Length::Fixed(spacing::MD)),
            action_buttons,
        ]
        .spacing(spacing::XS)
        .padding(spacing::XL)
        .max_width(700);

        let dialog_box = container(dialog_content)
            .style(theme::container_modal)
            .padding(spacing::SM)
            .width(Length::Fixed(700.0));

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
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::Name)
            )
            .label("Name")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Name))
            .text_size(typography::CAPTION),
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::Display)
            )
            .label("Display")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Display))
            .text_size(typography::CAPTION),
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::Trigger)
            )
            .label("Trigger")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Trigger))
            .text_size(typography::CAPTION),
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::Load)
            )
            .label("Load")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Load))
            .text_size(typography::CAPTION),
        ]
        .spacing(spacing::MD);

        // Row 2: Action, Animation, Conditions, Author Options
        let row2 = row![
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::Action)
            )
            .label("Actions")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Action))
            .text_size(typography::CAPTION),
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::Animation)
            )
            .label("Animations")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Animation))
            .text_size(typography::CAPTION),
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::Conditions)
            )
            .label("Conditions")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Conditions))
            .text_size(typography::CAPTION),
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::AuthorOptions)
            )
            .label("Author Options")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::AuthorOptions))
            .text_size(typography::CAPTION),
        ]
        .spacing(spacing::MD);

        // Row 3: Arrangement, Anchor, User Config, Metadata
        let row3 = row![
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::Arrangement)
            )
            .label("Arrangement")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Arrangement))
            .text_size(typography::CAPTION),
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::Anchor)
            )
            .label("Anchor")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Anchor))
            .text_size(typography::CAPTION),
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::UserConfig)
            )
            .label("User Config")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::UserConfig))
            .text_size(typography::CAPTION),
            checkbox(
                self.conflicts
                    .global_categories
                    .contains(&UpdateCategory::Metadata)
            )
            .label("Metadata")
            .on_toggle(|_| Message::ToggleGlobalCategory(UpdateCategory::Metadata))
            .text_size(typography::CAPTION),
        ]
        .spacing(spacing::MD);

        Column::new()
            .push(row1)
            .push(row2)
            .push(row3)
            .spacing(spacing::XS)
    }

    fn render_conflict_list(&self, conflicts: &[ImportConflict]) -> Column<'_, Message> {
        let mut list_col = Column::new().spacing(spacing::SM);

        for (idx, conflict) in conflicts.iter().enumerate() {
            let resolution = &self.conflicts.resolutions[idx];

            // Action dropdown using pick_list
            let action_options = vec![
                ConflictAction::Skip,
                ConflictAction::ReplaceAll,
                ConflictAction::UpdateSelected,
            ];

            let action_picker = pick_list(action_options, Some(resolution.action), move |action| {
                Message::SetConflictAction(idx, action)
            })
            .text_size(typography::CAPTION)
            .width(Length::Fixed(90.0));

            // Aura name with color based on action
            let name_color = match resolution.action {
                ConflictAction::Skip => colors::TEXT_MUTED,
                _ => colors::TEXT_PRIMARY,
            };

            let aura_id = conflict.aura_id.clone();
            let mut item_row = row![
                action_picker,
                text(aura_id).size(typography::BODY).color(name_color)
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center);

            // Group indicator
            if conflict.is_group {
                item_row = item_row.push(
                    text(format!("[Group: {} children]", conflict.child_count))
                        .color(colors::TEXT_MUTED)
                        .size(typography::CAPTION),
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
                        .size(typography::CAPTION),
                );
            }

            // Expand button for per-aura category selection
            if resolution.action == ConflictAction::UpdateSelected {
                let expand_text = if resolution.expanded { "v" } else { ">" };
                item_row = item_row.push(
                    button(text(expand_text).size(typography::CAPTION))
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
                .text_size(typography::MICRO),
            checkbox(resolution.categories.contains(&UpdateCategory::Display))
                .label("Display")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Display))
                .text_size(typography::MICRO),
            checkbox(resolution.categories.contains(&UpdateCategory::Trigger))
                .label("Trigger")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Trigger))
                .text_size(typography::MICRO),
            checkbox(resolution.categories.contains(&UpdateCategory::Load))
                .label("Load")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Load))
                .text_size(typography::MICRO),
        ]
        .spacing(spacing::SM);

        // Row 2: Action, Animation, Conditions, Author Options
        let row2 = row![
            space::horizontal().width(Length::Fixed(100.0)),
            checkbox(resolution.categories.contains(&UpdateCategory::Action))
                .label("Actions")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Action))
                .text_size(typography::MICRO),
            checkbox(resolution.categories.contains(&UpdateCategory::Animation))
                .label("Animations")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Animation))
                .text_size(typography::MICRO),
            checkbox(resolution.categories.contains(&UpdateCategory::Conditions))
                .label("Conditions")
                .on_toggle(move |_| Message::ToggleConflictCategory(
                    idx,
                    UpdateCategory::Conditions
                ))
                .text_size(typography::MICRO),
            checkbox(
                resolution
                    .categories
                    .contains(&UpdateCategory::AuthorOptions)
            )
            .label("Author Options")
            .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::AuthorOptions))
            .text_size(typography::MICRO),
        ]
        .spacing(spacing::SM);

        // Row 3: Arrangement, Anchor, User Config, Metadata
        let row3 = row![
            space::horizontal().width(Length::Fixed(100.0)),
            checkbox(resolution.categories.contains(&UpdateCategory::Arrangement))
                .label("Arrangement")
                .on_toggle(move |_| Message::ToggleConflictCategory(
                    idx,
                    UpdateCategory::Arrangement
                ))
                .text_size(typography::MICRO),
            checkbox(resolution.categories.contains(&UpdateCategory::Anchor))
                .label("Anchor")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Anchor))
                .text_size(typography::MICRO),
            checkbox(resolution.categories.contains(&UpdateCategory::UserConfig))
                .label("User Config")
                .on_toggle(move |_| Message::ToggleConflictCategory(
                    idx,
                    UpdateCategory::UserConfig
                ))
                .text_size(typography::MICRO),
            checkbox(resolution.categories.contains(&UpdateCategory::Metadata))
                .label("Metadata")
                .on_toggle(move |_| Message::ToggleConflictCategory(idx, UpdateCategory::Metadata))
                .text_size(typography::MICRO),
        ]
        .spacing(spacing::SM);

        Column::new()
            .push(row1)
            .push(row2)
            .push(row3)
            .spacing(2)
            .padding(Padding::default().bottom(spacing::SM))
    }

    /// Overlay the remove confirmation dialog on top of the main view
    pub(crate) fn overlay_remove_confirmation<'a>(
        &'a self,
        underlay: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let count = self.removal.pending_ids.len();

        // List of IDs to be removed
        let mut id_list = Column::new().spacing(spacing::XS);
        for id in &self.removal.pending_ids {
            id_list = id_list.push(
                text(id)
                    .size(typography::BODY)
                    .color(colors::TEXT_SECONDARY),
            );
        }

        let id_list_container = container(
            scrollable(id_list)
                .height(Length::Fixed(150.0))
                .style(theme::scrollable_style),
        )
        .style(theme::container_elevated)
        .padding(spacing::SM)
        .width(Length::Fill);

        let dialog_content = column![
            text(format!("Remove {} aura(s)?", count)).size(typography::HEADING),
            space::vertical().height(Length::Fixed(spacing::XS)),
            text("Groups will have all their children removed recursively.")
                .color(colors::TEXT_MUTED)
                .size(typography::CAPTION),
            space::vertical().height(Length::Fixed(spacing::SM)),
            id_list_container,
            space::vertical().height(Length::Fixed(spacing::MD)),
            row![
                button(text("Cancel").size(typography::BODY))
                    .style(theme::button_secondary)
                    .on_press(Message::HideRemoveConfirm),
                space::horizontal(),
                button(text("Remove").size(typography::BODY).color(colors::BG_VOID))
                    .style(theme::button_danger)
                    .on_press(Message::ConfirmRemoval),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
        ]
        .spacing(spacing::XS)
        .padding(spacing::XL)
        .max_width(450);

        let dialog_box = container(dialog_content)
            .style(theme::container_modal)
            .padding(spacing::SM)
            .width(Length::Fixed(450.0));

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
