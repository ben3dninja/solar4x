use bevy::{prelude::*, utils::HashMap};
use ratatui::{
    layout::Alignment,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{block::Title, Block, List, WidgetRef},
};

use crate::{
    app::body_id::BodyID,
    core_plugin::{BodyInfo, PrimaryBody},
    ui_plugin::space_map_plugin::FocusBody,
    utils::ui::Direction2,
};

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TreeViewEvent>()
            .add_systems(PostStartup, initialize_tree)
            .add_systems(
                Update,
                (
                    handle_tree_events,
                    update_focus_body.run_if(resource_exists::<FocusBody>),
                ),
            );
    }
}
#[derive(Debug, Event)]
pub enum TreeViewEvent {
    SelectTree(Direction2),
    ToggleTreeExpansion,
}

#[derive(Debug, Clone)]
struct TreeEntry {
    id: BodyID,
    name: String,
    is_last_child: bool,
    index_of_parent: Option<usize>,
    is_expanded: bool,
}

#[derive(Resource, Debug)]
pub struct TreeWidget {
    /// Indices of the entries in the system tree, and whether they are expanded or not
    visible_tree_entries: Vec<usize>,
    system_tree: Vec<TreeEntry>,
    focus_body: Option<BodyID>,
    selected_index: usize,
}

impl WidgetRef for TreeWidget {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let texts: Vec<Line<'_>> = self
            .visible_tree_entries
            .iter()
            .enumerate()
            .map(|(index, &index_in_tree)| {
                let entry = &self.system_tree[index_in_tree];
                let style = if Some(entry.id) == self.focus_body {
                    Style::default().bold()
                } else {
                    Style::default()
                };
                let prefix = String::from(if index == self.selected_index {
                    "> "
                } else {
                    "  "
                }) + &self.build_deepness_prefix(index_in_tree);
                vec![Span::from(prefix), Span::styled(entry.name.clone(), style)].into()
            })
            .collect();
        let list = List::new(texts)
            .block(
                Block::bordered()
                    .title(Title::from("Tree view".bold()).alignment(Alignment::Center)),
            )
            .highlight_symbol("> ");
        <List as WidgetRef>::render_ref(&list, area, buf)
    }
}

fn initialize_tree(
    mut commands: Commands,
    primary: Res<PrimaryBody>,
    focus_body: Option<Res<FocusBody>>,
    bodies: Query<&BodyInfo>,
) {
    #[derive(Clone)]
    struct Temp {
        children: Vec<BodyID>,
        semimajor_axis: i64,
        name: String,
    }
    let mut info: HashMap<BodyID, Temp> = bodies
        .iter()
        .map(|BodyInfo(data)| {
            (
                data.id,
                Temp {
                    children: data.orbiting_bodies.clone(),
                    semimajor_axis: data.semimajor_axis,
                    name: data.name.clone(),
                },
            )
        })
        .collect();
    let info_bis = info.clone();
    for entry in info.values_mut() {
        entry.children.retain(|b| info_bis.get(b).is_some());
        entry.children.sort_by_key(|b| info_bis[b].semimajor_axis);
    }
    fn fill_tree_rec(
        tree: &mut Vec<TreeEntry>,
        info: &HashMap<BodyID, Temp>,
        id: BodyID,
        index_of_parent: Option<usize>,
        is_last_child: bool,
    ) {
        let Temp { children, name, .. } = &info[&id];
        tree.push(TreeEntry {
            id,
            name: name.clone(),
            is_last_child,
            index_of_parent,
            is_expanded: false,
        });
        let index_of_parent = tree.len() - 1;
        let size = children.len();
        for (i, &child) in children.iter().enumerate() {
            fill_tree_rec(tree, info, child, Some(index_of_parent), i == size - 1);
        }
    }
    let mut system_tree = Vec::new();
    fill_tree_rec(&mut system_tree, &info, primary.0, None, true);

    commands.insert_resource(TreeWidget {
        system_tree,
        selected_index: 0,
        visible_tree_entries: vec![0],
        focus_body: focus_body.map(|r| r.0),
    });
}

fn handle_tree_events(mut tree_state: ResMut<TreeWidget>, mut reader: EventReader<TreeViewEvent>) {
    use Direction2::*;
    use TreeViewEvent::*;
    for event in reader.read() {
        match event {
            SelectTree(d) => match d {
                Down => tree_state.select_next_tree(),
                Up => tree_state.select_previous_tree(),
            },
            ToggleTreeExpansion => tree_state.toggle_selection_expansion(),
        }
    }
}

fn update_focus_body(mut tree_state: ResMut<TreeWidget>, focus_body: Res<FocusBody>) {
    tree_state.focus_body = Some(focus_body.0);
}

impl TreeWidget {
    /// Each entry of this vector corresponds to an ancestor of the body.
    /// The body itself is the last entry, and the primary body is the first.
    /// Each boolean corresponds to whether or not the ancestor is a last child
    /// so that we can decide which symbol we display
    pub fn compute_deepness_map(&self, index_in_tree: usize) -> Vec<bool> {
        let tree = &self.system_tree;
        let mut current = &tree[index_in_tree];
        let mut map = vec![current.is_last_child];
        while let Some(i) = current.index_of_parent {
            current = &tree[i];
            map.push(current.is_last_child);
        }
        map.reverse();
        map
    }

    fn build_deepness_prefix(&self, index_in_tree: usize) -> String {
        let mut prefix = String::new();
        let deepness_map = self.compute_deepness_map(index_in_tree);
        let last = deepness_map.len().saturating_sub(2);
        for (i, &is_last) in deepness_map.iter().skip(1).enumerate() {
            prefix.push_str(if is_last {
                if i == last {
                    "└─"
                } else {
                    "  "
                }
            } else if i == last {
                "├─"
            } else {
                "│ "
            });
        }
        prefix
    }

    /// Returns the index of the specified body in the system tree,
    /// or None if the body is not present
    pub fn index_of(&self, id: BodyID) -> Option<usize> {
        self.system_tree.iter().position(|entry| entry.id == id)
    }

    pub fn nth_visible_entry(&self, n: usize) -> Option<&TreeEntry> {
        self.visible_tree_entries
            .get(n)
            .map(|&i| &self.system_tree[i])
    }

    pub fn index_of_nth_visible_entry(&self, n: usize) -> Option<usize> {
        self.nth_visible_entry(n)
            .and_then(|entry| self.index_of(entry.id))
    }

    pub fn toggle_selection_expansion(&mut self) {
        self.toggle_entry_expansion(self.selected_index);
    }

    pub fn expand_entry_by_id(&mut self, id: BodyID) {
        if let Some(i) = self.index_of(id) {
            self.try_expand_entry(i);
        }
    }

    fn toggle_entry_expansion(&mut self, index: usize) {
        if index < self.visible_tree_entries.len() && !self.try_collapse_entry(index) {
            self.try_expand_entry(index);
        }
    }

    pub fn try_expand_entry(&mut self, index: usize) -> bool {
        let index_in_tree = self.visible_tree_entries[index];
        if self.system_tree[index_in_tree].is_expanded {
            return false;
        }
        self.system_tree[index_in_tree].is_expanded = true;
        let mut i = index_in_tree;
        let mut to_add = Vec::new();
        while let Some(parent) = self
            .system_tree
            .get_mut(i + 1)
            .and_then(|next| next.index_of_parent)
        {
            if parent < index_in_tree {
                break;
            }
            i += 1;
            if self.system_tree[parent].is_expanded {
                to_add.push(i);
            }
        }
        let end = self.visible_tree_entries.split_off(index + 1);
        self.visible_tree_entries.extend(to_add);
        self.visible_tree_entries.extend(end);
        true
    }

    pub fn try_collapse_entry(&mut self, index: usize) -> bool {
        let index_in_tree = self.visible_tree_entries[index];
        if !self.system_tree[index_in_tree].is_expanded {
            return false;
        }
        let mut i = 0;
        for next in self.system_tree[(index_in_tree + 1)..].iter() {
            if let Some(parent) = next.index_of_parent {
                if parent >= index_in_tree {
                    i += 1;
                    continue;
                }
            }
            break;
        }
        self.visible_tree_entries
            .drain((index + 1)..(index + i + 1));
        self.system_tree[index_in_tree].is_expanded = false;
        true
    }

    pub fn select_next_tree(&mut self) {
        self.selected_index = (self.selected_index + 1).min(self.visible_tree_entries.len() - 1)
    }

    pub fn select_previous_tree(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn selected_body_id(&self) -> BodyID {
        self.nth_visible_entry(self.selected_index).unwrap().id
    }

    pub fn select_body(&mut self, id: BodyID) -> bool {
        let mut ancestors = Vec::new();
        if let Some(index) = self.index_of(id) {
            let mut current = &self.system_tree[index];
            while let Some(parent_index_in_tree) = current.index_of_parent {
                ancestors.push(parent_index_in_tree);
                current = &self.system_tree[parent_index_in_tree];
            }
            ancestors.reverse();
            for index_in_tree in ancestors {
                if let Some(index) = self
                    .visible_tree_entries
                    .iter()
                    .position(|&i| i == index_in_tree)
                {
                    self.try_expand_entry(index);
                }
            }
            if let Some(index) = self
                .visible_tree_entries
                .iter()
                .position(|&index_in_tree| self.system_tree[index_in_tree].id == id)
            {
                self.selected_index = index;
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use crate::{app::body_data::BodyType, core_plugin::CorePlugin};

    use super::{TreePlugin, TreeWidget};

    #[test]
    fn test_initialize_tree() {
        let mut app = App::new();
        app.add_plugins((CorePlugin::default(), TreePlugin));
        app.update();
        let world = &app.world;
        let tree = world.resource::<TreeWidget>();
        assert_eq!(tree.system_tree.len(), 9);
        assert!(tree.system_tree[0].is_last_child);
        assert!(tree.system_tree[8].is_last_child);
        for i in 1..8 {
            assert!(!tree.system_tree[i].is_last_child);
        }
    }
    #[test]
    fn test_select_body() {
        let mut app = App::new();
        app.add_plugins((CorePlugin::default(), TreePlugin));
        app.update();
        let world = &mut app.world;
        let mut tree = world.resource_mut::<TreeWidget>();
        let earth = "terre".into();
        tree.select_body(earth);
        assert_eq!(tree.selected_body_id(), earth)
    }

    #[test]
    fn test_toggle_entry_expansion() {
        let mut app = App::new();
        app.add_plugins((CorePlugin::default(), TreePlugin));
        app.update();
        let mut tree = app.world.resource_mut::<TreeWidget>();
        tree.toggle_selection_expansion();
        assert_eq!(tree.visible_tree_entries.len(), 9);
        assert!(tree.nth_visible_entry(0).unwrap().is_expanded);
        for i in 1..9 {
            tree.toggle_entry_expansion(i);
        }
        assert_eq!(tree.visible_tree_entries.len(), 9);
        tree.toggle_selection_expansion();
        assert_eq!(tree.visible_tree_entries.len(), 1);
        assert!(!tree.nth_visible_entry(0).unwrap().is_expanded);
    }

    #[test]
    fn test_deepness_map() {
        let mut app = App::new();
        app.add_plugins((
            CorePlugin {
                smallest_body_type: BodyType::Moon,
            },
            TreePlugin,
        ));
        app.update();
        let world = &app.world;
        let tree_state = world.resource::<TreeWidget>();
        assert_eq!(
            tree_state.compute_deepness_map(tree_state.index_of("lune".into()).unwrap()),
            vec![true, false, true]
        );
    }

    #[test]
    fn test_build_deepness_prefix() {
        let mut app = App::new();
        app.add_plugins((
            CorePlugin {
                smallest_body_type: BodyType::Moon,
            },
            TreePlugin,
        ));
        app.update();
        let mut tree_state = app.world.resource_mut::<TreeWidget>();
        assert_eq!(tree_state.build_deepness_prefix(0), "");
        dbg!(&tree_state.visible_tree_entries);
        tree_state.toggle_selection_expansion();
        dbg!(&tree_state.visible_tree_entries);
        dbg!(tree_state.compute_deepness_map(tree_state.index_of_nth_visible_entry(8).unwrap()));
        assert_eq!(
            tree_state.build_deepness_prefix(tree_state.index_of_nth_visible_entry(8).unwrap()),
            "└─"
        );
        tree_state.toggle_entry_expansion(8);
        assert_eq!(
            tree_state.build_deepness_prefix(tree_state.index_of_nth_visible_entry(9).unwrap()),
            "  ├─"
        );
    }
}
