use bevy::{
    app::AppExit,
    ecs::{
        event::EventWriter,
        system::{In, Res},
    },
};
use bevy_ratatui::error::exit_on_error;
use color_eyre::eyre::Result;

use crate::client_plugin::Testing;
pub fn exit_on_error_if_app(
    input: In<Result<()>>,
    app_exit: EventWriter<AppExit>,
    testing: Option<Res<Testing>>,
) {
    if testing.is_some() {
        input.0.unwrap();
    } else {
        exit_on_error(input, app_exit);
    }
}
