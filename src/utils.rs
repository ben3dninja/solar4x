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

#[derive(Debug, Clone, Copy)]
pub enum Direction4 {
    Front,
    Back,
    Left,
    Right,
}
