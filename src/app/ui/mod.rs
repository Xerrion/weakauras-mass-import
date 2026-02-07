//! UI rendering components for iced.

mod dialogs;
mod main_panel;
mod sidebar;

use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Element, Length};

use crate::theme::{self, colors, spacing, typography};

use super::{Message, WeakAuraImporter};

impl WeakAuraImporter {
    /// Render the menu buttons for the header
    pub(crate) fn render_menu_buttons(&self) -> Element<'_, Message> {
        let file_menu = button(text("File").size(typography::BODY))
            .style(theme::button_frameless)
            .on_press(Message::LoadFromFile);

        let edit_menu = button(text("Paste").size(typography::BODY))
            .style(theme::button_frameless)
            .on_press(Message::PasteFromClipboard);

        let clear_btn = button(text("Clear").size(typography::BODY))
            .style(theme::button_frameless)
            .on_press(Message::ClearInput);

        let view_label = if self.show_decoded_view {
            "Hide JSON"
        } else {
            "Show JSON"
        };
        let view_menu = button(text(view_label).size(typography::BODY))
            .style(theme::button_frameless)
            .on_press(Message::ToggleDecodedView);

        let setup_label = if self.selected_sv_path.is_some() {
            "Change Install"
        } else {
            "Select Install"
        };
        let setup_btn = button(text(setup_label).size(typography::BODY))
            .style(theme::button_frameless)
            .on_press(Message::ShowSetupWizard);

        row![file_menu, edit_menu, clear_btn, view_menu, setup_btn]
            .spacing(spacing::SM)
            .align_y(iced::Alignment::Center)
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

        let mut content = row![status_text]
            .spacing(spacing::MD)
            .align_y(iced::Alignment::Center);

        // Add progress bar if loading or importing
        if self.is_loading || self.is_importing {
            let progress = if self.is_importing {
                self.import_progress
            } else {
                self.loading_progress
            };

            use iced::widget::progress_bar;
            use iced::Border;

            content = content.push(space::horizontal().width(Length::Fill));
            content = content.push(
                container(
                    progress_bar(0.0..=1.0, progress)
                        .style(|_theme| progress_bar::Style {
                            background: colors::BG_SURFACE.into(),
                            bar: colors::GOLD.into(),
                            border: Border::default().rounded(4.0),
                        })
                )
                .height(8.0)
                .width(Length::Fixed(200.0))
            );
        } else {
            content = content.push(space::horizontal().width(Length::Fill));
        }

        container(content)
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
                    text(json).size(typography::CAPTION)
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
