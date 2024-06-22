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
        app.insert_resource(CoreConfig::from(self))
            .add_systems(Startup, (build_system).chain());
    }
}

#[derive(Resource)]
pub struct PrimaryBody(pub BodyID);

#[derive(Resource)]
pub struct EntityMapping {
    pub id_mapping: HashMap<BodyID, Entity>,
}

#[derive(Component)]
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

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use crate::{app::body_data::BodyType, core_plugin::EntityMapping};

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
    }
}
