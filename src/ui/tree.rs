use crate::{
    app::body_id::BodyID,
    utils::list::{select_next_clamp, select_previous_clamp},
};

use super::UiState;

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub id: BodyID,
    pub is_expanded: bool,
    pub deepness: usize,
}

impl TreeEntry {
    pub fn new_main_body(id: BodyID) -> Self {
        Self {
            id,
            is_expanded: false,
            deepness: 0,
        }
    }

    pub fn create_children(&self, children_ids: impl Iterator<Item = BodyID>) -> Vec<TreeEntry> {
        children_ids
            .map(|id| Self {
                id,
                is_expanded: false,
                deepness: self.deepness + 1,
            })
            .collect()
    }
}

impl UiState {
    pub fn toggle_selection_expansion(&mut self) {
        if let Some(sel_id) = self.tree_state.selected() {
            self.toggle_entry_expansion(sel_id);
        }
    }

    pub fn expand_entry_by_id(&mut self, id: BodyID) {
        if let Some(index) = self.tree_entries.iter().position(|entry| entry.id == id) {
            self.expand_entry(index);
        }
    }

    fn toggle_entry_expansion(&mut self, index: usize) {
        let entry = &self.tree_entries[index];
        if entry.is_expanded {
            self.collapse_entry(index)
        } else {
            self.expand_entry(index)
        }
    }

    pub fn expand_entry(&mut self, index: usize) {
        let entry = &self.tree_entries[index];
        if entry.is_expanded {
            return;
        }
        let bodies = &self.shared_info.bodies;
        let mut children: Vec<_> = bodies[&entry.id]
            .orbiting_bodies
            .clone()
            .into_iter()
            .filter(|id| bodies.contains_key(id))
            .collect();
        children.sort_by(|a, b| bodies[a].semimajor_axis.cmp(&bodies[b].semimajor_axis));
        let children = entry.create_children(children.into_iter());
        let end = self.tree_entries.split_off(index + 1);
        self.tree_entries.extend(children);
        self.tree_entries.extend(end);
        self.tree_entries[index].is_expanded = true;
    }

    pub fn collapse_entry(&mut self, index: usize) {
        let entry = &self.tree_entries[index];
        if !entry.is_expanded {
            return;
        }
        let deepness = entry.deepness;
        let mut i = 0;
        for next in self.tree_entries[(index + 1)..].iter() {
            if next.deepness <= deepness {
                break;
            }
            i += 1;
        }
        self.tree_entries.drain((index + 1)..(index + i + 1));
        self.tree_entries[index].is_expanded = false;
    }

    pub fn select_next_tree(&mut self) {
        select_next_clamp(&mut self.tree_state, self.tree_entries.len() - 1)
    }

    pub fn select_previous_tree(&mut self) {
        select_previous_clamp(&mut self.tree_state, 0)
    }

    pub fn selected_body_id_tree(&self) -> BodyID {
        self.tree_entries[self.tree_state.selected().unwrap_or_default()].id
    }

    pub fn entry_is_last_child(&self, index: usize) -> Option<bool> {
        if self.tree_entries.get(index + 1).is_none() {
            Some(true)
        } else {
            self.tree_entries
                .get(index)
                .map(|entry1| entry1.deepness > self.tree_entries[index + 1].deepness)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::App;

    #[test]
    fn test_toggle_entry_expansion() {
        let (_, mut ui) = App::new_simple_testing().unwrap();
        ui.toggle_selection_expansion();
        assert_eq!(ui.tree_entries.len(), 9);
        assert!(ui.tree_entries[0].is_expanded);
        for i in 1..9 {
            assert_eq!(ui.tree_entries[i].deepness, 1);
        }
        for i in 1..9 {
            ui.toggle_entry_expansion(i);
            ui.toggle_entry_expansion(i);
        }
        ui.toggle_selection_expansion();
        assert_eq!(ui.tree_entries.len(), 1);
        assert!(!ui.tree_entries[0].is_expanded);
    }

    #[test]
    fn test_entry_is_last_child() {
        let (_, mut ui) = App::new_simple_testing().unwrap();
        ui.toggle_selection_expansion();
        for i in 0..8 {
            assert!(!ui.entry_is_last_child(i).unwrap());
        }
        assert!(ui.entry_is_last_child(8).unwrap())
    }
}
