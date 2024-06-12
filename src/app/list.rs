use nalgebra::{SimdPartialOrd, SimdValue};

use crate::bodies::body_id::BodyID;

use super::App;

#[derive(Debug, Clone)]
pub struct ListEntry {
    pub id: BodyID,
    pub is_expanded: bool,
    pub deepness: usize,
}

impl ListEntry {
    pub fn new_main_body(id: BodyID) -> Self {
        Self {
            id,
            is_expanded: false,
            deepness: 0,
        }
    }

    pub fn create_children(&self, children_ids: impl Iterator<Item = BodyID>) -> Vec<ListEntry> {
        children_ids
            .map(|id| Self {
                id,
                is_expanded: false,
                deepness: self.deepness + 1,
            })
            .collect()
    }
}

impl App {
    pub fn toggle_selection_expansion(&mut self) -> Result<(), String> {
        let sel_id = self
            .list_state
            .selected()
            .ok_or("no selected body".to_owned())?;
        self.toggle_entry_expansion(sel_id);
        Ok(())
    }

    fn toggle_entry_expansion(&mut self, index: usize) {
        let entry = &self.listed_bodies[index];
        if entry.is_expanded {
            let deepness = entry.deepness;
            let mut i = 1;
            for next in self.listed_bodies[(index + 1)..].iter() {
                if next.deepness == deepness {
                    break;
                }
                i += 1
            }
            self.listed_bodies = self.listed_bodies.drain((index + 1)..(index + i)).collect();
        } else {
            let bodies = self.system.borrow().bodies[&entry.id]
                .orbiting_bodies
                .clone();
            let children = entry.create_children(bodies.into_iter());
            let end = self.listed_bodies.split_off(index + 1);
            self.listed_bodies.extend(children);
            self.listed_bodies.extend(end);
        }
        let entry = &mut self.listed_bodies[index];
        entry.is_expanded = !entry.is_expanded;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_toggle_entry_expansion() {}
}
