use ratatui::widgets::ListState;

use super::ui::Direction2;

pub fn select_next_clamp(list_state: &mut ListState, max: usize) {
    list_state.select(match list_state.selected() {
        Some(i) if i == max => Some(i),
        Some(i) => Some(i + 1),
        None => Some(max),
    })
}

pub fn select_previous_clamp(list_state: &mut ListState, min: usize) {
    list_state.select(match list_state.selected() {
        Some(i) if i == min => Some(i),
        Some(i) => Some(i - 1),
        None => Some(min),
    })
}

pub trait ClampedList {
    fn list_state(&mut self) -> &mut ListState;

    fn len(&mut self) -> usize;

    fn select_next(&mut self) {
        let len = self.len();
        select_next_clamp(&mut self.list_state(), len - 1);
    }

    fn select_previous(&mut self) {
        select_previous_clamp(&mut self.list_state(), 0);
    }

    fn select_adjacent(&mut self, direction: Direction2) {
        match direction {
            Direction2::Up => self.select_previous(),
            Direction2::Down => self.select_next(),
        }
    }
}
