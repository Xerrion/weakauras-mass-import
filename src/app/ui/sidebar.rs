//! Sidebar rendering: SavedVariables selection and existing aura tree.

use iced::widget::{button, column, container, row, scrollable, space, text, text_input, Column};
use iced::{Element, Length};

use crate::saved_variables::AuraTreeNode;
use crate::theme::{self, colors};

use super::super::{Message, WeakAuraImporter};

impl WeakAuraImporter {
    pub(crate) fn render_sidebar(&self) -> Element<'_, Message> {
        let mut content = Column::new().spacing(8).padding(12);

        // Step 1 Header
        content = content.push(
            text("Step 1: Select SavedVariables")
                .size(16)
                .color(colors::GOLD),
        );

        // WoW path input
        let wow_path_input = text_input("WoW Path...", &self.wow_path)
            .on_input(Message::WowPathChanged)
            .style(theme::text_input_style);

        let browse_btn = button(text("..."))
            .style(theme::button_secondary)
            .on_press(Message::BrowseWowPath);

        content = content.push(
            row![
                text("WoW Path:").color(colors::TEXT_SECONDARY),
                wow_path_input.width(Length::Fill),
                browse_btn,
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        );

        // Discovered files list
        content = content.push(text("Discovered files:").color(colors::TEXT_PRIMARY));

        let files_list = if self.discovered_sv_files.is_empty() {
            column![text("No SavedVariables found").color(colors::TEXT_MUTED)]
        } else {
            let mut files_col = Column::new().spacing(4);
            for sv_info in &self.discovered_sv_files {
                let is_selected = self.selected_sv_path.as_ref() == Some(&sv_info.path);
                let text_color = if is_selected {
                    colors::GOLD
                } else {
                    colors::TEXT_SECONDARY
                };

                let label_text = format!("{} ({})", sv_info.account, sv_info.flavor);
                let path_clone = sv_info.path.clone();

                let file_btn = button(text(label_text).color(text_color))
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

        let files_container = container(scrollable(files_list).height(Length::Fixed(150.0)))
            .style(theme::container_elevated)
            .padding(8);

        content = content.push(files_container);

        // Manual selection button
        content = content.push(
            button(text("Select file manually..."))
                .style(theme::button_secondary)
                .on_press(Message::SelectSavedVariablesManually),
        );

        // Selected file info
        if let Some(path) = &self.selected_sv_path {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            content = content.push(text("Selected:").color(colors::TEXT_SECONDARY));
            content = content.push(text(filename.to_string()).color(colors::SUCCESS));
        }

        // Scanning indicator
        if self.is_scanning {
            content = content.push(
                row![
                    text("Loading...").color(colors::TEXT_SECONDARY),
                    text(&self.scanning_message).color(colors::TEXT_MUTED),
                ]
                .spacing(8),
            );
        }

        // Existing auras tree
        if !self.existing_auras_tree.is_empty() && !self.is_scanning {
            content = content.push(
                row![
                    text("Existing Auras:").color(colors::TEXT_PRIMARY),
                    text(format!("({})", self.existing_auras_count)).color(colors::TEXT_MUTED),
                ]
                .spacing(8),
            );

            // Expand/Collapse buttons
            content = content.push(
                row![
                    button(text("Expand all").size(12))
                        .style(theme::button_secondary)
                        .on_press(Message::ExpandAllGroups),
                    button(text("Collapse all").size(12))
                        .style(theme::button_secondary)
                        .on_press(Message::CollapseAllGroups),
                ]
                .spacing(4),
            );

            // Selection & removal controls
            let mut selection_row = row![
                button(text("Select all").size(12))
                    .style(theme::button_secondary)
                    .on_press(Message::SelectAllForRemoval),
                button(text("Deselect all").size(12))
                    .style(theme::button_secondary)
                    .on_press(Message::DeselectAllForRemoval),
            ]
            .spacing(4);

            if !self.selected_for_removal.is_empty() {
                let count = self.selected_for_removal.len();
                selection_row = selection_row.push(
                    button(text(format!("Remove ({})", count)).size(12))
                        .style(theme::button_danger)
                        .on_press(Message::ShowRemoveConfirm),
                );
            }

            content = content.push(selection_row);

            // Scrollable aura tree
            let tree_content = self.render_aura_tree();
            let tree_container = container(scrollable(tree_content).height(Length::Fixed(200.0)))
                .style(theme::container_elevated)
                .padding(8);

            content = content.push(tree_container);
        }

        // Import result
        if let Some(result) = &self.last_import_result {
            content = content.push(text("Last import:").color(colors::TEXT_PRIMARY));
            content = content.push(text(result.summary()).color(colors::TEXT_SECONDARY));
        }

        container(scrollable(content))
            .width(Length::Fixed(280.0))
            .height(Length::Fill)
            .style(theme::container_panel)
            .into()
    }

    fn render_aura_tree(&self) -> Column<'_, Message> {
        let mut tree_col = Column::new().spacing(2);

        for node in &self.existing_auras_tree {
            tree_col = self.render_aura_tree_node(tree_col, node, 0);
        }

        tree_col
    }

    fn render_aura_tree_node<'a>(
        &self,
        mut col: Column<'a, Message>,
        node: &AuraTreeNode,
        depth: usize,
    ) -> Column<'a, Message> {
        let indent = depth as u16 * 12;

        let is_selected = self.selected_for_removal.contains(&node.id);

        // Checkbox for removal selection
        let checkbox_text = if is_selected { "[x]" } else { "[ ]" };
        let checkbox_btn = button(text(checkbox_text).size(12))
            .style(theme::button_frameless)
            .on_press(Message::ToggleAuraForRemoval(node.id.clone()));

        let mut node_row = row![]
            .spacing(4)
            .padding(iced::Padding::default().left(indent as f32));

        node_row = node_row.push(checkbox_btn);

        if node.is_group {
            let is_expanded = self.expanded_groups.contains(&node.id);
            let expand_icon = if is_expanded { "v" } else { ">" };

            let expand_btn = button(text(expand_icon).size(12))
                .style(theme::button_frameless)
                .on_press(Message::ToggleGroupExpanded(node.id.clone()));

            node_row = node_row.push(expand_btn);
            node_row = node_row.push(text(node.id.clone()).color(colors::GOLD).size(13));
            node_row = node_row.push(
                text(format!("({})", node.total_count() - 1))
                    .color(colors::TEXT_MUTED)
                    .size(12),
            );
        } else {
            node_row = node_row.push(space::horizontal().width(Length::Fixed(18.0)));
            node_row = node_row.push(text(node.id.clone()).color(colors::TEXT_SECONDARY).size(13));
        }

        col = col.push(node_row);

        // Render children if expanded
        if node.is_group && self.expanded_groups.contains(&node.id) {
            for child in &node.children {
                col = self.render_aura_tree_node(col, child, depth + 1);
            }
        }

        col
    }
}
