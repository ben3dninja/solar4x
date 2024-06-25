use std::time::Duration;

use bevy::{
    app::{AppExit, FixedMain},
    prelude::*,
    time::TimePlugin,
    utils::HashMap,
};

use crate::{
    app::{
        body_data::{BodyData, BodyType},
        body_id::BodyID,
    },
    engine_plugin::{EllipticalOrbit, GameSpeed, Position, ToggleTime},
    utils::{de::read_main_bodies, ui::Direction2},
};
pub struct CorePlugin {
    pub smallest_body_type: BodyType,
}

impl Default for CorePlugin {
    fn default() -> Self {
        CorePlugin {
            smallest_body_type: BodyType::Planet,
        }
    }
}

#[derive(Resource)]
pub struct CoreConfig {
    smallest_body_type: BodyType,
}

impl From<&CorePlugin> for CoreConfig {
    fn from(value: &CorePlugin) -> Self {
        Self {
            smallest_body_type: value.smallest_body_type,
        }
    }
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MinimalPlugins)
            .insert_resource(CoreConfig::from(self))
            .add_event::<CoreEvent>()
            .add_systems(Startup, (build_system).chain())
            .add_systems(Update, handle_core_events);
    }
}

#[derive(Resource)]
pub struct PrimaryBody(pub BodyID);

#[derive(Resource)]
pub struct EntityMapping {
    pub id_mapping: HashMap<BodyID, Entity>,
}

#[derive(Component, Debug)]
pub struct BodyInfo(pub BodyData);

pub fn build_system(mut commands: Commands, config: Res<CoreConfig>) {
    let bodies = read_main_bodies()
        .expect("Failed to read bodies")
        .into_iter()
        .filter(|data| data.body_type <= config.smallest_body_type);
    let primary_body = bodies
        .clone()
        .find(|data| data.host_body.is_none())
        .expect("no primary body found")
        .id;
    commands.insert_resource(PrimaryBody(primary_body));
    let mut id_mapping = HashMap::new();
    for data in bodies {
        let id = data.id;
        let entity = commands.spawn((
            Position::default(),
            EllipticalOrbit::from(&data),
            BodyInfo(data),
        ));
        id_mapping.insert(id, entity.id());
    }
    commands.insert_resource(EntityMapping { id_mapping });
}

#[derive(Event)]
pub enum CoreEvent {
    Quit,
    EngineSpeed(Direction2),
    ToggleTime,
}

fn handle_core_events(
    mut reader: EventReader<CoreEvent>,
    mut quit_writer: EventWriter<AppExit>,
    mut toggle_time: ResMut<ToggleTime>,
    mut speed: ResMut<GameSpeed>,
) {
    for event in reader.read() {
        match event {
            CoreEvent::Quit => {
                quit_writer.send_default();
            }
            CoreEvent::EngineSpeed(d) => match d {
                Direction2::Up => speed.0 *= 1.5,
                Direction2::Down => speed.0 /= 1.5,
            },
            CoreEvent::ToggleTime => toggle_time.0 = !toggle_time.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use crate::{
        app::body_data::BodyType, core_plugin::EntityMapping, engine_plugin::EllipticalOrbit,
    };

    use super::{BodyInfo, CorePlugin};

    #[test]
    fn test_build_system() {
        let mut app = App::new();
        app.add_plugins(CorePlugin {
            smallest_body_type: BodyType::Planet,
        });
        app.update();
        let mut world = app.world;
        assert_eq!(world.resource::<EntityMapping>().id_mapping.len(), 9);
        assert_eq!(world.query::<&BodyInfo>().iter(&world).len(), 9);
        let (orbit, BodyInfo(data)) = world
            .query::<(&EllipticalOrbit, &BodyInfo)>()
            .iter(&world)
            .find(|(_, BodyInfo(data))| data.id == "terre".into())
            .unwrap();
        assert_eq!(orbit.semimajor_axis, 149598023.);
        assert_eq!(data.semimajor_axis, 149598023);
    }
}
