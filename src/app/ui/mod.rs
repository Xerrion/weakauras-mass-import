//! UI rendering components for iced.

mod dialogs;
mod main_panel;
mod sidebar;

use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Element, Length};

use crate::theme::{self, colors, spacing, typography};

use super::{Message, WeakAuraImporter};

impl WeakAuraImporter {
    /// Render the menu bar
    pub(crate) fn render_menu_bar(&self) -> Element<'_, Message> {
        let file_menu = button(text("File").size(typography::BODY))
            .style(theme::button_secondary)
            .on_press(Message::LoadFromFile);

        let edit_menu = button(text("Paste").size(typography::BODY))
            .style(theme::button_secondary)
            .on_press(Message::PasteFromClipboard);

        let clear_btn = button(text("Clear").size(typography::BODY))
            .style(theme::button_secondary)
            .on_press(Message::ClearInput);

        let view_label = if self.show_decoded_view {
            "Hide JSON"
        } else {
            "Show JSON"
        };
        let view_menu = button(text(view_label).size(typography::BODY))
            .style(theme::button_secondary)
            .on_press(Message::ToggleDecodedView);

        let menu_row = row![
            file_menu,
            edit_menu,
            clear_btn,
            view_menu,
            space::horizontal()
        ]
        .spacing(spacing::SM)
        .padding(spacing::SM);

        container(menu_row)
            .width(Length::Fill)
            .style(theme::container_toolbar)
            .into()
    }

    /// Render the status bar
    pub(crate) fn render_status_bar(&self) -> Element<'_, Message> {
        let status_color = if self.status_is_error {
            colors::ERROR
        } else {
            colors::TEXT_SECONDARY
        };

        let status_text = text(&self.status_message)
            .size(typography::CAPTION)
            .color(status_color);

        container(status_text)
            .width(Length::Fill)
            .padding(spacing::SM)
            .style(theme::container_status_bar)
            .into()
    }

    /// Render the decoded JSON panel (right side)
    pub(crate) fn render_decoded_panel(&self) -> Element<'_, Message> {
        let content = if let Some(idx) = self.selected_aura_index {
            if let Some(entry) = self.parsed_auras.get(idx) {
                if let Some(aura) = &entry.aura {
                    let json = serde_json::to_string_pretty(&aura.data)
                        .unwrap_or_else(|_| "Failed to serialize".to_string());
                    text(json).size(typography::SMALL)
                } else {
                    text("No aura data")
                        .size(typography::BODY)
                        .color(colors::TEXT_MUTED)
                }
            } else {
                text("Invalid selection")
                    .size(typography::BODY)
                    .color(colors::TEXT_MUTED)
            }
        } else {
            text("Select an aura to view decoded data")
                .size(typography::BODY)
                .color(colors::TEXT_MUTED)
        };

        let header = text("Decoded Data")
            .size(typography::HEADING)
            .color(colors::GOLD);

        let panel_content = column![header, content]
            .spacing(spacing::SM)
            .padding(spacing::MD)
            .width(Length::Fixed(300.0));

        container(scrollable(panel_content).style(theme::scrollable_style))
            .width(Length::Fixed(300.0))
            .height(Length::Fill)
            .style(theme::container_panel)
            .into()
    }
}
