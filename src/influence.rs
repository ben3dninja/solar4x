use bevy::{math::DVec3, prelude::*};

use crate::{
    bodies::body_id::BodyID,
    core_plugin::{build_system, BodiesMapping, BodyInfo, LoadingState, PrimaryBody},
    engine_plugin::Position,
    leapfrog::LeapfrogSystems,
    main_game::GameStage,
};

pub struct InfluencePlugin;

impl Plugin for InfluencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(LoadingState::Loading),
            setup_hill_spheres.after(build_system),
        )
        .add_systems(
            FixedUpdate,
            update_influence
                .before(LeapfrogSystems)
                .run_if(in_state(GameStage::Action)),
        );
    }
}

#[derive(Component, Clone, Copy)]
pub struct Mass(pub f64);

#[derive(Component, Clone, Copy)]
pub struct HillRadius(pub f64);

/// Component storing the bodies that influence the object's trajectory
#[derive(Component, Default, Debug)]
pub struct Influenced {
    pub main_influencer: Option<Entity>,
    pub influencers: Vec<Entity>,
}

pub fn setup_hill_spheres(
    mut commands: Commands,
    query: Query<&BodyInfo>,
    primary: Query<(Entity, &BodyInfo), With<PrimaryBody>>,
    mapping: Res<BodiesMapping>,
) {
    let mut queue = vec![(primary.single().1 .0.id, 0.)];
    let mut i = 0;
    while i < queue.len() {
        let (id, parent_mass) = queue[i];
        if let Some(entity) = mapping.0.get(&id) {
            if let Ok(BodyInfo(data)) = query.get(*entity) {
                let radius = (data.semimajor_axis
                    * (1. - data.eccentricity)
                    * (data.mass / (3. * (parent_mass + data.mass))).powf(1. / 3.))
                .max(data.radius);
                commands.entity(*entity).insert(HillRadius(radius));
                queue.extend(data.orbiting_bodies.iter().map(|c| (*c, data.mass)));
            }
        }
        i += 1;
    }
    commands
        .entity(primary.single().0)
        .insert(HillRadius(f64::INFINITY));
}

pub fn update_influence(
    mut influenced: Query<(&Position, &mut Influenced)>,
    bodies: Query<(&Position, &HillRadius, &BodyInfo)>,
    mapping: Res<BodiesMapping>,
    main_body: Query<&BodyInfo, With<PrimaryBody>>,
) {
    let main_body = main_body.single().0.id;
    influenced
        .par_iter_mut()
        .for_each(|(object_pos, mut influence)| {
            *influence = compute_influence(object_pos, &bodies, mapping.as_ref(), main_body);
        });
}

pub fn compute_influence(
    Position(object_pos): &Position,
    bodies: &Query<(&Position, &HillRadius, &BodyInfo)>,
    mapping: &BodiesMapping,
    main_body: BodyID,
) -> Influenced {
    // if an object is not in a bodie's sphere of influence, it is not in its children's either
    fn influencers_rec(
        body: BodyID,
        query: &Query<(&Position, &HillRadius, &BodyInfo)>,
        mapping: &BodiesMapping,
        object_pos: &DVec3,
        influences: &mut Vec<(Entity, f64)>,
    ) {
        if let Some(e) = mapping.0.get(&body) {
            let (Position(body_pos), HillRadius(hill_radius), BodyInfo(data)) =
                query.get(*e).unwrap();
            let r = *object_pos - *body_pos;
            let dist = r.length();
            if dist < *hill_radius {
                influences.push((*e, *hill_radius));
                data.orbiting_bodies.iter().for_each(|child| {
                    influencers_rec(*child, query, mapping, object_pos, influences);
                })
            }
        }
    }

    let mut influences = Vec::new();
    influencers_rec(main_body, bodies, mapping, object_pos, &mut influences);
    Influenced {
        main_influencer: influences
            .iter()
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|a| a.0),
        influencers: influences.into_iter().map(|a| a.0).collect(),
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use crate::{
        bodies::{bodies_config::BodiesConfig, body_data::BodyType, body_id::id_from},
        client_plugin::{ClientMode, ClientPlugin},
        core_plugin::BodiesMapping,
        engine_plugin::{Position, Velocity},
        main_game::ShipEvent,
        spaceship::ShipInfo,
        utils::algebra::circular_orbit_around_body,
    };

    use super::{Influenced, Mass};

    #[test]
    fn test_influence() {
        let mut app = App::new();
        app.add_plugins(
            ClientPlugin::testing()
                .with_bodies(BodiesConfig::SmallestBodyType(BodyType::Moon))
                .in_mode(ClientMode::Singleplayer),
        );
        app.update();
        let world = app.world_mut();
        let mapping = &world.resource::<BodiesMapping>().0;
        let moon = mapping[&id_from("lune")];
        let earth = mapping[&id_from("terre")];
        let sun = mapping[&id_from("soleil")];
        let (mass, pos, speed) = world
            .query::<(&Mass, &Position, &Velocity)>()
            .get(world, moon)
            .unwrap();
        let (spawn_pos, spawn_speed) = circular_orbit_around_body(100., mass.0, pos.0, speed.0);
        world.send_event(ShipEvent::Create(ShipInfo {
            id: id_from("s"),
            spawn_pos,
            spawn_speed,
        }));
        app.update();
        let world = app.world_mut();
        let influenced = world.query::<&Influenced>().single(world);

        assert!(influenced.influencers.contains(&moon));
        assert!(influenced.influencers.contains(&earth));
        assert!(influenced.influencers.contains(&sun));
        assert_eq!(influenced.main_influencer, Some(moon));
        assert_eq!(influenced.influencers.len(), 3);
    }
}