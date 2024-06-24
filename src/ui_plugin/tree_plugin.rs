use bevy::prelude::*;
use ratatui::widgets::ListState;

use crate::{core_plugin::BodyInfo, ui::tree::TreeEntry};

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, initialize_tree);
    }
}

#[derive(Resource)]
pub struct UiTreeState {
    tree_entries: Vec<TreeEntry>,
    tree_state: ListState,
}

fn initialize_tree(mut commands: Commands, query: Query<&BodyInfo>) {}
