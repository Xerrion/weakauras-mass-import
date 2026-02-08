//! Sidebar rendering: existing aura tree.

use iced::widget::{button, checkbox, column, container, row, scrollable, space, text, Column};
use iced::{Element, Length};

use crate::saved_variables::AuraTreeNode;
use crate::theme::{self, colors, spacing, typography};

use super::super::{Message, WeakAuraImporter};

impl WeakAuraImporter {
    pub(crate) fn render_sidebar(&self) -> Element<'_, Message> {
        let mut content = Column::new()
            .spacing(spacing::SM)
            .padding(spacing::SM)
            .width(Length::Fill);

        // Existing Auras header with all buttons in one row
        let header_row = row![
            text("Existing Auras")
                .size(typography::HEADING)
                .color(colors::GOLD),
            text(format!("({})", self.saved_vars.auras_count))
                .size(typography::BODY)
                .color(colors::TEXT_MUTED),
        ]
        .spacing(spacing::XS)
        .align_y(iced::Alignment::Center);

        content = content.push(header_row);

        // Existing auras tree
        if !self.saved_vars.auras_tree.is_empty() && !self.tasks.is_scanning {
            // All controls in a single row
            let mut controls_row = row![
                button(text("Expand").size(typography::CAPTION))
                    .style(theme::button_secondary)
                    .on_press(Message::ExpandAllGroups),
                button(text("Collapse").size(typography::CAPTION))
                    .style(theme::button_secondary)
                    .on_press(Message::CollapseAllGroups),
                button(text("Select").size(typography::CAPTION))
                    .style(theme::button_secondary)
                    .on_press(Message::SelectAllForRemoval),
                button(text("Deselect").size(typography::CAPTION))
                    .style(theme::button_secondary)
                    .on_press(Message::DeselectAllForRemoval),
            ]
            .spacing(spacing::XS);

            if !self.removal.selected_ids.is_empty() {
                let count = self.removal.selected_ids.len();
                controls_row = controls_row.push(
                    button(text(format!("Remove ({})", count)).size(typography::CAPTION))
                        .style(theme::button_danger)
                        .on_press(Message::ShowRemoveConfirm),
                );
            }

            content = content.push(controls_row);

            // Scrollable aura tree
            let tree_content = self.render_aura_tree();
            let tree_container = container(
                scrollable(tree_content)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .style(theme::scrollable_style),
            )
            .style(theme::container_inset)
            .padding(spacing::SM)
            .width(Length::Fill)
            .height(Length::Fill);

            content = content.push(tree_container);
        } else if self.tasks.is_scanning {
            content = content.push(
                text("Loading...")
                    .size(typography::BODY)
                    .color(colors::TEXT_MUTED),
            );
        } else {
            content = content.push(
                text("No auras found")
                    .size(typography::BODY)
                    .color(colors::TEXT_MUTED),
            );
        }

        // Import result
        if let Some(result) = &self.status.last_import_result {
            content = content.push(
                container(
                    column![
                        text("Last import:")
                            .size(typography::CAPTION)
                            .color(colors::TEXT_PRIMARY),
                        text(result.summary())
                            .size(typography::CAPTION)
                            .color(colors::TEXT_SECONDARY),
                    ]
                    .spacing(spacing::XS),
                )
                .padding(spacing::SM)
                .style(theme::container_surface)
                .width(Length::Fill),
            );
        }

        container(content)
            .width(Length::Fixed(self.sidebar.width))
            .height(Length::Fill)
            .style(theme::container_elevated)
            .into()
    }

    fn render_aura_tree(&self) -> Column<'_, Message> {
        let mut tree_col = Column::new().spacing(2).width(Length::Fill);

        for node in &self.saved_vars.auras_tree {
            tree_col = self.render_aura_tree_node(tree_col, node, 0);
        }

        tree_col
    }

    fn render_aura_tree_node<'a>(
        &self,
        mut col: Column<'a, Message>,
        node: &'a AuraTreeNode,
        depth: usize,
    ) -> Column<'a, Message> {
        let indent = depth as u16 * 12;

        let is_selected = self.removal.selected_ids.contains(&node.id);

        // Checkbox for removal selection
        let node_id = node.id.clone();
        let checkbox_btn = checkbox(is_selected)
            .on_toggle(move |_| Message::ToggleAuraForRemoval(node_id.clone()));

        let mut node_row = row![]
            .spacing(spacing::XS)
            .padding(iced::Padding::default().left(indent as f32))
            .width(Length::Fill);

        node_row = node_row.push(checkbox_btn);

        if node.is_group {
            let is_expanded = self.sidebar.expanded_groups.contains(&node.id);
            let expand_icon = if is_expanded { "▼" } else { "▶" };

            let expand_btn = button(text(expand_icon).size(typography::CAPTION))
                .style(theme::button_frameless)
                .on_press(Message::ToggleGroupExpanded(node.id.clone()));

            node_row = node_row.push(expand_btn);
            node_row = node_row.push(text(&node.id).size(typography::BODY).color(colors::GOLD));
            node_row = node_row.push(
                text(format!("({})", node.total_count() - 1))
                    .color(colors::TEXT_MUTED)
                    .size(typography::CAPTION),
            );
        } else {
            node_row = node_row.push(space::horizontal().width(Length::Fixed(18.0)));
            node_row = node_row.push(
                text(&node.id)
                    .size(typography::BODY)
                    .color(colors::TEXT_SECONDARY),
            );
        }

        col = col.push(node_row);

        // Render children if expanded
        if node.is_group && self.sidebar.expanded_groups.contains(&node.id) {
            for child in &node.children {
                col = self.render_aura_tree_node(col, child, depth + 1);
            }
        }

        col
    }
}
