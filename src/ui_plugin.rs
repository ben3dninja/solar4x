use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        block::Title,
        canvas::{Canvas, Circle},
        Block, Borders, Clear, List, ListState, Paragraph, Widget,
    },
    Frame,
};

use crate::utils::ui::centered_rect;

mod space_map_plugin;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiTreeState)
            .insert_resource(UiSearchState)
            .add_systems(Update, (update_search_entries, render));
    }
}

#[derive(Resource)]
struct UiTreeState {
    tree_entries: Vec<TreeEntry>,
    tree_state: ListState,
}

#[derive(Resource)]
struct UiSearchState {
    search_entries: Vec<BodyID>,
    search_state: ListState,
    search_character_index: usize,
    search_input: String,
    search_matcher: SkimMatcherV2,
}

#[derive(Default, Copy, Clone, Resource)]
pub enum AppScreen {
    #[default]
    Main,
    Info,
}

#[derive(Default, Copy, Clone, Resource)]
pub enum ExplorerMode {
    #[default]
    Tree,
    Search,
}

// TODO : only on key event
fn update_search_entries(mut state: ResMut<UiSearchState>) {
    state.search_entries = state.search(&state.search_input);
    if state.search_state.selected().is_none() && !state.search_entries.is_empty() {
        state.search_state.select(Some(0));
    }
}

fn render(
    mut ctx: ResMut<RatatuiContext>,
    mut tree_state: ResMut<UiTreeState>,
    search_state: Res<UiSearchState>,
) {
    ctx.draw(|f| {
        let chunks =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Fill(1)]).split(f.size());
        match self.get_explorer_mode() {
            ExplorerMode::Tree => self.draw_tree(f, chunks[0]),
            ExplorerMode::Search => self.draw_search(f, chunks[0]),
        }
        self.draw_canvas(f, chunks[1]);
        if matches!(self.get_current_screen(), AppScreen::Info) {
            self.draw_popup(f);
        }
    })
}
fn draw_tree(&mut self, f: &mut Frame, rect: Rect) {
    let texts: Vec<Line<'_>> = self
        .tree_entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            self.shared_info.bodies.get(&entry.id).map(|body| {
                let style = if body.id == self.focus_body {
                    Style::default().bold()
                } else {
                    Style::default()
                };
                let deepness_marker = Span::from(if entry.deepness == 0 {
                    String::new()
                } else {
                    "│ ".repeat(entry.deepness.saturating_sub(1))
                        + if self.entry_is_last_child(index).unwrap() {
                            "└─"
                        } else {
                            "├─"
                        }
                });
                vec![deepness_marker, Span::styled(body.name.clone(), style)].into()
            })
        })
        .collect();
    let list = List::new(texts)
        .block(
            Block::bordered().title(Title::from("Tree view".bold()).alignment(Alignment::Center)),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, rect, &mut self.tree_state);
}

fn draw_search(&mut self, f: &mut Frame, rect: Rect) {
    let names: Vec<_> = self
        .search_entries
        .iter()
        .filter_map(|entry| {
            self.shared_info
                .bodies
                .get(entry)
                .map(|body| body.name.clone())
        })
        .collect();
    let texts: Vec<Text> = names
        .into_iter()
        .map(|s| Text::styled(s, Style::default()))
        .collect();
    let search_bar = Paragraph::new(&self.search_input[..]).block(Block::bordered());
    let list = List::new(texts)
        .block(
            Block::bordered().title(Title::from("Search view".bold()).alignment(Alignment::Center)),
        )
        .highlight_symbol("> ");
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(rect);
    f.render_widget(search_bar, chunks[0]);
    f.render_stateful_widget(list, chunks[1], &mut self.search_state);
}

fn draw_popup(f: &mut Frame, main_body_info: BodyInfo) {
    let main_body = state
        .shared_info
        .bodies
        .get(&self.selected_body_id_tree())
        .unwrap();
    let popup_block = Block::default()
        .title(&main_body.name[..])
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    let area = centered_rect(25, 25, f.size());
    Clear.render(area, f.buffer_mut());
    let info = Paragraph::new(format!(
        "Body type: {}\n\
            N of orbiting bodies: {}\n\
            Radius: {} km\n\
            Revolution period: {} earth days",
        main_body.body_type,
        main_body.orbiting_bodies.len(),
        main_body.radius,
        main_body.revolution_period,
    ))
    .block(popup_block);
    f.render_widget(info, area);
}
