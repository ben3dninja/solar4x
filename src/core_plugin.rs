use bevy::{prelude::*, utils::HashMap};

use crate::{
    app::{
        body_data::{BodyData, BodyType},
        body_id::BodyID,
    },
    engine_plugin::{EllipticalOrbit, Position},
    utils::de::read_main_bodies,
};
pub struct CorePlugin {
    smallest_body_type: BodyType,
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
        app.insert_resource(CoreConfig::from(self))
            .add_systems(Startup, (build_system_info, spawn_bodies).chain());
    }
}

#[derive(Resource)]
pub struct SystemInfo {
    pub bodies_data: HashMap<BodyID, BodyData>,
}

#[derive(Resource)]
pub struct PrimaryBody(pub BodyID);

#[derive(Resource)]
pub struct EntityMapping {
    pub id_mapping: HashMap<BodyID, Entity>,
}

fn build_system_info(mut commands: Commands, config: Res<CoreConfig>) {
    let bodies = read_main_bodies()
        .expect("Failed to read bodies")
        .into_iter()
        .filter(|data| data.body_type <= config.smallest_body_type);
    let bodies_data: HashMap<_, _> = bodies.into_iter().map(|data| (data.id, data)).collect();
    let primary_body = bodies_data
        .values()
        .find(|data| data.host_body.is_none())
        .expect("no primary body found")
        .id;
    commands.insert_resource(SystemInfo { bodies_data });
    commands.insert_resource(PrimaryBody(primary_body));
}

fn spawn_bodies(mut commands: Commands, system: Res<SystemInfo>) {
    let mut id_mapping = HashMap::new();
    for (id, data) in &system.bodies_data {
        let entity = commands.spawn((Position::default(), EllipticalOrbit::from(data)));
        id_mapping.insert(*id, entity.id());
    }
    commands.insert_resource(EntityMapping { id_mapping });
}
