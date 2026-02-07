//! Main content panel: input area, aura list, import controls.

use iced::widget::{
    button, checkbox, container, progress_bar, row, scrollable, space, text, text_input, Column,
};
use iced::{Element, Length};

use crate::theme::{self, colors, spacing, typography};

use super::super::{Message, WeakAuraImporter};

impl WeakAuraImporter {
    pub(crate) fn render_main_content(&self) -> Element<'_, Message> {
        let mut content = Column::new().spacing(spacing::SM).padding(spacing::MD);

        // Step 2 Header
        content = content.push(
            text("Step 2: Load WeakAuras")
                .size(typography::HEADING)
                .color(colors::GOLD),
        );

        // Action Buttons row
        let paste_btn = if self.show_paste_input {
            button(
                text("Paste")
                    .size(typography::BODY)
                    .color(colors::BG_DARKEST),
            )
            .style(theme::button_primary)
            .on_press(Message::TogglePasteInput)
        } else {
            button(text("Paste").size(typography::BODY))
                .style(theme::button_secondary)
                .on_press(Message::TogglePasteInput)
        };

        let load_file_btn = if self.is_loading {
            button(text("Load file").size(typography::BODY)).style(theme::button_secondary)
        } else {
            button(text("Load file").size(typography::BODY))
                .style(theme::button_secondary)
                .on_press(Message::LoadFromFile)
        };

        let load_folder_btn = if self.is_loading {
            button(text("Load folder").size(typography::BODY)).style(theme::button_secondary)
        } else {
            button(text("Load folder").size(typography::BODY))
                .style(theme::button_secondary)
                .on_press(Message::LoadFromFolder)
        };

        let clear_btn = button(text("Clear").size(typography::BODY))
            .style(theme::button_secondary)
            .on_press(Message::ClearInput);

        content = content.push(
            row![paste_btn, load_file_btn, load_folder_btn, clear_btn]
                .spacing(spacing::SM)
                .align_y(iced::Alignment::Center),
        );

        // Loading progress bar (shown during async file/folder loading)
        if self.is_loading {
            content = content.push(progress_bar(0.0..=1.0, self.loading_progress));
            if !self.loading_message.is_empty() {
                content = content.push(
                    text(&self.loading_message)
                        .size(typography::CAPTION)
                        .color(colors::TEXT_SECONDARY),
                );
            }
        }

        // Paste input area (only shown when toggled)
        if self.show_paste_input {
            content = content.push(self.render_paste_input_area());
        }

        // Step 3: Review & Import (only if auras parsed)
        if !self.parsed_auras.is_empty() {
            content = content.push(space::vertical().height(Length::Fixed(spacing::LG)));
            content = content.push(
                container(text(""))
                    .height(Length::Fixed(1.0))
                    .width(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(colors::BORDER.into()),
                        ..Default::default()
                    }),
            );
            content = content.push(space::vertical().height(Length::Fixed(spacing::LG)));
            content = content.push(self.render_review_import_section());
        }

        container(scrollable(content).style(theme::scrollable_style))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::container_panel)
            .into()
    }

    fn render_paste_input_area(&self) -> Element<'_, Message> {
        let mut paste_content = Column::new().spacing(spacing::SM);

        // Text input area
        let input_area = text_input(
            "Paste WeakAura import strings here (one per line)",
            &self.input_text,
        )
        .on_input(Message::InputTextChanged)
        .style(theme::text_input_style)
        .width(Length::Fill);

        let input_container = container(input_area)
            .style(theme::container_elevated)
            .padding(spacing::SM);

        paste_content = paste_content.push(input_container);

        // Paste from clipboard and Parse buttons
        let paste_clipboard_btn = button(text("Paste from clipboard").size(typography::BODY))
            .style(theme::button_secondary)
            .on_press(Message::PasteFromClipboard);

        let parse_btn = button(
            text("Parse")
                .size(typography::BODY)
                .color(colors::BG_DARKEST),
        )
        .style(theme::button_primary)
        .on_press(Message::ParseInput);

        paste_content = paste_content.push(
            row![paste_clipboard_btn, space::horizontal(), parse_btn]
                .spacing(spacing::SM)
                .align_y(iced::Alignment::Center),
        );

        paste_content.into()
    }

    fn render_review_import_section(&self) -> Element<'_, Message> {
        let mut content = Column::new().spacing(spacing::SM);

        // Header
        content = content.push(
            text("Step 3: Review & Import")
                .size(typography::HEADING)
                .color(colors::GOLD),
        );

        // Check if we can import
        let can_import = self.selected_sv_path.is_some()
            && self
                .parsed_auras
                .iter()
                .any(|e| e.selected && e.validation.is_valid)
            && !self.is_importing
            && !self.is_loading;

        // Selection Controls, Import Button & Stats
        let select_all_btn = if self.is_importing {
            button(text("Select All").size(typography::BODY)).style(theme::button_secondary)
        } else {
            button(text("Select All").size(typography::BODY))
                .style(theme::button_secondary)
                .on_press(Message::SelectAllAuras)
        };

        let deselect_all_btn = if self.is_importing {
            button(text("Deselect All").size(typography::BODY)).style(theme::button_secondary)
        } else {
            button(text("Deselect All").size(typography::BODY))
                .style(theme::button_secondary)
                .on_press(Message::DeselectAllAuras)
        };

        // Remove Selected button
        let has_selected = self.parsed_auras.iter().any(|e| e.selected);
        let remove_selected_btn = if has_selected && !self.is_importing && !self.is_loading {
            button(text("Remove Selected").size(typography::BODY))
                .style(theme::button_secondary)
                .on_press(Message::RemoveSelectedFromList)
        } else {
            button(text("Remove Selected").size(typography::BODY)).style(theme::button_secondary)
        };

        // Import button
        let import_btn = if can_import {
            button(
                text("Import Selected >>")
                    .size(typography::BODY)
                    .color(colors::BG_DARKEST),
            )
            .style(theme::button_primary)
            .on_press(Message::ShowImportConfirm)
        } else {
            button(
                text("Import Selected >>")
                    .size(typography::BODY)
                    .color(colors::TEXT_MUTED),
            )
            .style(theme::button_secondary)
        };

        // Stats
        let selected_count = self.parsed_auras.iter().filter(|e| e.selected).count();
        let valid_count = self
            .parsed_auras
            .iter()
            .filter(|e| e.validation.is_valid)
            .count();

        let stats_format = format!(
            "{} selected / {} valid / {} total",
            selected_count,
            valid_count,
            self.parsed_auras.len()
        );

        let controls_row = if !can_import && self.selected_sv_path.is_none() && !self.is_importing {
            row![
                button(text("Select All").size(typography::BODY))
                    .style(theme::button_secondary)
                    .on_press(Message::SelectAllAuras),
                button(text("Deselect All").size(typography::BODY))
                    .style(theme::button_secondary)
                    .on_press(Message::DeselectAllAuras),
                button(text("Remove Selected").size(typography::BODY))
                    .style(theme::button_secondary),
                button(
                    text("Import Selected >>")
                        .size(typography::BODY)
                        .color(colors::TEXT_MUTED)
                )
                .style(theme::button_secondary),
                text("Select a SavedVariables file first")
                    .size(typography::BODY)
                    .color(colors::TEXT_MUTED),
                space::horizontal(),
                text(stats_format)
                    .size(typography::CAPTION)
                    .color(colors::TEXT_SECONDARY)
            ]
            .spacing(spacing::SM)
            .align_y(iced::Alignment::Center)
        } else {
            row![
                select_all_btn,
                deselect_all_btn,
                remove_selected_btn,
                import_btn,
                space::horizontal(),
                text(stats_format)
                    .size(typography::CAPTION)
                    .color(colors::TEXT_SECONDARY)
            ]
            .spacing(spacing::SM)
            .align_y(iced::Alignment::Center)
        };

        content = content.push(controls_row);

        // Progress bar (shown during import)
        if self.is_importing {
            content = content.push(
                container(progress_bar(0.0..=1.0, self.import_progress)).height(Length::Fixed(8.0)),
            );
            if !self.import_progress_message.is_empty() {
                content = content.push(
                    text(&self.import_progress_message)
                        .size(typography::CAPTION)
                        .color(colors::TEXT_SECONDARY),
                );
            }
        }

        // Aura List
        content = content.push(self.render_aura_list());

        content.into()
    }

    fn render_aura_list(&self) -> Element<'_, Message> {
        let mut list_col = Column::new().spacing(spacing::XS);

        for (idx, entry) in self.parsed_auras.iter().enumerate() {
            let is_selected_for_view = self.selected_aura_index == Some(idx);
            let is_valid = entry.validation.is_valid;

            // Remove button
            let remove_btn = button(text("X").color(colors::ERROR).size(typography::CAPTION))
                .style(theme::button_frameless)
                .on_press(Message::RemoveAuraFromList(idx));

            // Checkbox for selection
            let checkbox_widget =
                checkbox(entry.selected).on_toggle(move |_| Message::ToggleAuraSelection(idx));

            // Aura Info Label
            let summary = entry.validation.summary();
            let label_color = if is_valid {
                if is_selected_for_view {
                    colors::GOLD_LIGHT
                } else {
                    colors::TEXT_PRIMARY
                }
            } else {
                colors::TEXT_MUTED
            };

            let label_btn = button(text(summary).size(typography::BODY).color(label_color))
                .style(if is_selected_for_view {
                    theme::button_primary
                } else {
                    theme::button_frameless
                })
                .on_press(Message::SelectAuraForPreview(idx));

            let mut item_row = row![remove_btn]
                .spacing(spacing::SM)
                .align_y(iced::Alignment::Center);

            if is_valid {
                item_row = item_row.push(checkbox_widget);
            } else {
                // Placeholder to maintain alignment
                item_row = item_row.push(space::horizontal().width(Length::Fixed(24.0)));
            }

            item_row = item_row.push(label_btn);

            // Group Child Count
            if entry.validation.is_group {
                item_row = item_row.push(
                    text(format!(
                        "[Group: {} children]",
                        entry.validation.child_count
                    ))
                    .color(colors::TEXT_MUTED)
                    .size(typography::CAPTION),
                );
            }

            // Source file (if loaded from folder)
            if let Some(ref source) = entry.source_file {
                item_row = item_row.push(
                    text(format!("< {}", source))
                        .color(colors::TEXT_MUTED)
                        .size(typography::CAPTION),
                );
            }

            list_col = list_col.push(item_row);
        }

        let list_container = container(
            scrollable(list_col)
                .height(Length::Fill)
                .style(theme::scrollable_style),
        )
        .style(theme::container_elevated)
        .padding(spacing::SM)
        .height(Length::Fill);

        list_container.into()
    }
}
