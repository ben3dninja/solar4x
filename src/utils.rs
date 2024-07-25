pub mod algebra;
pub mod args;
pub mod de;
pub mod ecs;
pub mod hash;
pub mod list;
pub mod ui;

#[derive(Debug, Clone, Copy)]
pub enum Direction2 {
    Up,
    Down,
}

impl From<&Direction2> for isize {
    fn from(value: &Direction2) -> Self {
        match value {
            Direction2::Up => 1,
            Direction2::Down => -1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction4 {
    Front,
    Back,
    Left,
    Right,
}
