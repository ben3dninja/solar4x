use bevy::prelude::*;
use ratatui::{
    buffer::Buffer,
    widgets::{Block, Borders, Paragraph, WidgetRef},
};

use crate::{
    bodies::body_data::BodyData,
    core_plugin::{AppState, BodyInfo, EntityMapping, GameSet, PrimaryBody},
};

use super::{
    tree_plugin::{initialize_tree, ChangeSelectionEvent, TreeState},
    UiInitSet,
};

pub struct InfoPlugin;

impl Plugin for InfoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InfoToggle(false))
            .add_systems(
                OnEnter(AppState::Game),
                initialize_info.in_set(UiInitSet).after(initialize_tree),
            )
            .add_systems(
                Update,
                update_info
                    .in_set(GameSet)
                    .run_if(on_event::<ChangeSelectionEvent>()),
            );
    }
}

#[derive(Resource)]
pub struct InfoToggle(pub bool);

#[derive(Resource)]
pub struct InfoWidget {
    body_info: BodyData,
}

impl WidgetRef for InfoWidget {
    fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let body_info = &self.body_info;
        let info = Paragraph::new(format!(
            "Body type: {}\n\
            N of orbiting bodies: {}\n\
            Radius: {} km\n\
            Revolution period: {} earth days",
            body_info.body_type,
            body_info.orbiting_bodies.len(),
            body_info.radius,
            body_info.revolution_period,
        ))
        .block(
            Block::default()
                .title(&body_info.name[..])
                .borders(Borders::ALL),
        );
        info.render_ref(area, buf);
    }
}

fn initialize_info(mut commands: Commands, primary: Query<&BodyInfo, With<PrimaryBody>>) {
    commands.insert_resource(InfoWidget {
        body_info: primary.single().0.clone(),
    });
}

fn update_info(
    mut widget: ResMut<InfoWidget>,
    tree: Option<Res<TreeState>>,
    bodies: Query<&BodyInfo>,
    mapping: Res<EntityMapping>,
) {
    if let Some(tree) = tree {
        let id = tree.selected_body_id();
        if let Ok(body_info) = bodies.get(mapping.id_mapping[&id]) {
            widget.body_info = body_info.0.clone();
        }
    }
}
