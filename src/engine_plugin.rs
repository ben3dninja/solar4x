use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
};

use crate::{
    bodies::body_data::BodyData,
    core_plugin::{build_system, AppState, BodyInfo, EntityMapping, GameSet, PrimaryBody},
    utils::{
        algebra::{degs, mod_180, rads},
        ui::Direction2,
    },
};

// Speed in days per second
const DEFAULT_SPEED: f64 = 10.;
pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EngineEvent>()
            .insert_resource(GameTime::default())
            .insert_resource(GameSpeed::default())
            .insert_resource(ToggleTime(true))
            .add_systems(
                OnEnter(AppState::Game),
                (update_local, update_global).chain().after(build_system),
            )
            .add_systems(
                FixedUpdate,
                (update_time, update_local, update_global)
                    .in_set(GameSet)
                    .chain()
                    .run_if(resource_equals(ToggleTime(true))),
            )
            .add_systems(Update, handle_engine_events.in_set(GameSet));
    }
}

#[derive(Resource, PartialEq)]
pub struct ToggleTime(pub bool);

#[derive(Resource, Default)]
pub struct GameTime(pub f64);

#[derive(Resource)]
pub struct GameSpeed(pub f64);

impl Default for GameSpeed {
    fn default() -> Self {
        GameSpeed(DEFAULT_SPEED)
    }
}

#[derive(Event)]
pub enum EngineEvent {
    EngineSpeed(Direction2),
    ToggleTime,
}

fn handle_engine_events(
    mut reader: EventReader<EngineEvent>,
    mut toggle_time: ResMut<ToggleTime>,
    mut speed: ResMut<GameSpeed>,
) {
    use EngineEvent::*;
    for event in reader.read() {
        match event {
            EngineSpeed(d) => match d {
                Direction2::Up => speed.0 *= 1.5,
                Direction2::Down => speed.0 /= 1.5,
            },
            ToggleTime => toggle_time.0 = !toggle_time.0,
        }
    }
}

pub fn update_time(mut game_time: ResMut<GameTime>, speed: Res<GameSpeed>, app_time: Res<Time>) {
    game_time.0 += speed.0 * app_time.delta_seconds_f64();
}

pub fn update_local(mut orbits: Query<&mut EllipticalOrbit>, time: Res<GameTime>) {
    orbits.par_iter_mut().for_each(|mut o| o.update_pos(time.0));
}

pub fn update_global(
    mut query: Query<(&mut Position, &EllipticalOrbit, &BodyInfo)>,
    primary: Res<PrimaryBody>,
    mapping: Res<EntityMapping>,
) {
    let mut queue = vec![(primary.0, DVec3::ZERO)];
    let mut i = 0;
    while i < queue.len() {
        let (id, parent_pos) = queue[i];
        if let Some(entity) = mapping.id_mapping.get(&id) {
            if let Ok((mut world_pos, orbit, info)) = query.get_mut(*entity) {
                let pos = parent_pos + orbit.local_pos;
                world_pos.0 = pos;
                queue.extend(info.0.orbiting_bodies.iter().map(|c| (*c, pos)));
            }
        }
        i += 1;
    }
}

#[derive(Component, Default, Debug)]
pub struct Position(pub DVec3);

#[derive(Component, Default, Clone, Debug)]
pub struct EllipticalOrbit {
    eccentricity: f64,
    pub semimajor_axis: f64,
    inclination: f64,
    long_asc_node: f64,
    arg_periapsis: f64,
    initial_mean_anomaly: f64,
    revolution_period: f64,

    mean_anomaly: f64,
    eccentric_anomaly: f64,
    /// 2D position in the orbital plane around the host body
    orbital_position: DVec2,
    // orbital_velocity: DVec2,
    /// 3D position with respect to the host body
    pub local_pos: DVec3,
}

const E_TOLERANCE: f64 = 1e-6;
// see https://ssd.jpl.nasa.gov/planets/approx_pos.html
#[allow(non_snake_case)]
impl EllipticalOrbit {
    fn update_M(&mut self, time: f64) {
        if self.revolution_period == 0. {
            return;
        }
        self.mean_anomaly =
            mod_180(self.initial_mean_anomaly + 360. * time / self.revolution_period);
    }
    fn update_E(&mut self, time: f64) {
        self.update_M(time);
        let M = self.mean_anomaly;
        let e = self.eccentricity;
        let ed = degs(e);
        let mut E = M + ed * rads(M).sin();
        // TODO : change formulas to use radians instead
        let mut dM = M - (E - ed * rads(E).sin());
        let mut dE = dM / (1. - e * rads(E).cos());
        for _ in 0..10 {
            if dE.abs() <= E_TOLERANCE {
                break;
            }
            dM = M - (E - ed * rads(E).sin());
            dE = dM / (1. - e * rads(E).cos());
            E += dE;
        }
        self.eccentric_anomaly = E;
    }
    fn update_orb_pos(&mut self, time: f64) {
        self.update_E(time);
        let a = self.semimajor_axis;
        let E = rads(self.eccentric_anomaly);
        let e = self.eccentricity;
        let x = a * (E.cos() - e);
        let y = a * (1. - e * e).sqrt() * E.sin();
        self.orbital_position = DVec2::new(x, y);
    }

    pub fn update_pos(&mut self, time: f64) {
        self.update_orb_pos(time);
        let x = self.orbital_position.x;
        let y = self.orbital_position.y;
        let o = rads(self.arg_periapsis);
        let O = rads(self.long_asc_node);
        let I = rads(self.inclination);
        let x_glob = (o.cos() * O.cos() - o.sin() * O.sin() * I.cos()) * x
            + (-o.sin() * O.cos() - o.cos() * O.sin() * I.cos()) * y;
        let y_glob = (o.cos() * O.sin() + o.sin() * O.cos() * I.cos()) * x
            + (-o.sin() * O.sin() + o.cos() * O.cos() * I.cos()) * y;
        let z_glob = (o.sin() * I.sin()) * x + (o.cos() * I.sin()) * y;
        self.local_pos = DVec3::new(x_glob, y_glob, z_glob);
    }
}

impl From<&BodyData> for EllipticalOrbit {
    fn from(data: &BodyData) -> Self {
        Self {
            eccentricity: data.eccentricity,
            semimajor_axis: data.semimajor_axis as f64,
            inclination: data.inclination,
            long_asc_node: data.long_asc_node,
            arg_periapsis: data.arg_periapsis,
            initial_mean_anomaly: data.initial_mean_anomaly,
            revolution_period: data.revolution_period,
            mean_anomaly: data.initial_mean_anomaly,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use crate::{
        bodies::body_data::BodyType,
        core_plugin::{BodiesConfig, BodyInfo},
        engine_plugin::{EllipticalOrbit, EnginePlugin, Position},
        standalone_plugin::StandalonePlugin,
    };

    #[test]
    fn test_update_local() {
        let mut app = App::new();
        app.add_plugins((
            StandalonePlugin(BodiesConfig::SmallestBodyType(BodyType::Planet)),
            EnginePlugin,
        ));
        app.update();
        let mut world = app.world;
        let mut query = world.query::<(&EllipticalOrbit, &BodyInfo)>();
        let (orbit, _) = query
            .iter(&world)
            .find(|(_, BodyInfo(data))| data.id == "terre".into())
            .unwrap();
        let earth_dist = orbit.local_pos.length();
        assert!(147095000. <= earth_dist);
        assert!(earth_dist <= 152100000.);
    }

    #[test]
    fn test_update_global() {
        let mut app = App::new();
        app.add_plugins((
            StandalonePlugin(BodiesConfig::SmallestBodyType(BodyType::Moon)),
            EnginePlugin,
        ));
        app.update();
        let mut world = app.world;
        let mut query = world.query::<(&Position, &BodyInfo)>();
        let (&Position(moon_pos), _) = query
            .iter(&world)
            .find(|(_, BodyInfo(data))| data.id == "lune".into())
            .unwrap();
        let moon_length = moon_pos.length();
        let min = 147095000. - 405500.;
        let max = 152100000. + 405500.;
        assert!(min <= moon_length);
        assert!(moon_length <= max)
    }
}
