use ratatui::widgets::ListState;

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
