use std::f64::consts::PI;

use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
};

use crate::{
    bodies::body_data::BodyData,
    core_plugin::{
        build_system, AppState, BodyInfo, EntityMapping, PrimaryBody, SimulationSet, SystemInitSet,
    },
    utils::{
        algebra::{degs, mod_180, rads, rotate},
        ui::Direction2,
    },
};

// Speed in days per second
const DEFAULT_SPEED: f64 = 10.;
pub const SECONDS_PER_DAY: f64 = 86400.;

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EngineEvent>()
            .insert_resource(GameTime::default())
            .insert_resource(GameSpeed::default())
            .insert_resource(ToggleTime(false))
            .add_systems(
                OnEnter(AppState::Loaded),
                (update_local, update_global)
                    .chain()
                    .after(build_system)
                    .in_set(SystemInitSet),
            )
            .add_systems(
                FixedUpdate,
                (update_time, update_local, update_global)
                    .in_set(SimulationSet)
                    .chain()
                    .run_if(resource_equals(ToggleTime(true))),
            )
            .add_systems(Update, handle_engine_events.in_set(SimulationSet));
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

#[derive(Event, Clone, Copy)]
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
    mut query: Query<(&mut Position, &mut Velocity, &EllipticalOrbit, &BodyInfo)>,
    primary: Query<&BodyInfo, With<PrimaryBody>>,
    mapping: Res<EntityMapping>,
) {
    let mut queue = vec![(primary.single().0.id, (DVec3::ZERO, DVec3::ZERO))];
    let mut i = 0;
    while i < queue.len() {
        let (id, (parent_pos, parent_velocity)) = queue[i];
        if let Some(entity) = mapping.id_mapping.get(&id) {
            if let Ok((mut world_pos, mut world_velocity, orbit, info)) = query.get_mut(*entity) {
                let pos = parent_pos + orbit.local_pos;
                let velocity = parent_velocity + orbit.local_velocity;
                world_pos.0 = pos;
                world_velocity.0 = velocity;
                queue.extend(info.0.orbiting_bodies.iter().map(|c| (*c, (pos, velocity))));
            }
        }
        i += 1;
    }
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Position(pub DVec3);

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Velocity(pub DVec3);

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
    orbital_velocity: DVec2,
    /// 3D position with respect to the host body (in kilometers)
    pub local_pos: DVec3,
    /// 3D velocity (in kilometers per day)
    pub local_velocity: DVec3,
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
        if self.revolution_period == 0. {
            return;
        }
        let Mdot = 2. * PI / self.revolution_period;
        let Edot = Mdot / (1. - e * E.cos());
        let Pdot = -a * (E.sin()) * Edot;
        let Qdot = a * (E.cos()) * Edot * (1. - e * e).sqrt();
        self.orbital_velocity = DVec2::new(Pdot, Qdot);
    }

    pub fn update_pos(&mut self, time: f64) {
        self.update_orb_pos(time);
        let o = rads(self.arg_periapsis);
        let O = rads(self.long_asc_node);
        let I = rads(self.inclination);
        self.local_pos = rotate(self.orbital_position, o, O, I);
        self.local_velocity = rotate(self.orbital_velocity, o, O, I);
    }
}

impl From<&BodyData> for EllipticalOrbit {
    fn from(data: &BodyData) -> Self {
        Self {
            eccentricity: data.eccentricity,
            semimajor_axis: data.semimajor_axis,
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
    use bevy::{app::App, prelude::NextState};

    use crate::{
        bodies::body_data::BodyType,
        client_plugin::{ClientMode, ClientPlugin},
        core_plugin::{BodiesConfig, BodyInfo},
        engine_plugin::{EllipticalOrbit, EnginePlugin, Position},
    };

    #[test]
    fn test_update_local() {
        let mut app = App::new();
        app.add_plugins((
            ClientPlugin::testing(BodiesConfig::SmallestBodyType(BodyType::Planet)),
            EnginePlugin,
        ));
        app.world
            .resource_mut::<NextState<ClientMode>>()
            .set(ClientMode::Explorer);
        app.update();
        let mut world = app.world;
        let mut query = world.query::<(&EllipticalOrbit, &BodyInfo)>();
        let (orbit, _) = query
            .iter(&world)
            .find(|(_, BodyInfo(data))| data.id == "terre".into())
            .unwrap();
        let (earth_dist, earth_speed) = (orbit.local_pos.length(), orbit.local_velocity.length());
        assert!(147095000. <= earth_dist);
        assert!(earth_dist <= 152100000.);
        assert!((earth_speed / 24. - 107200.).abs() <= 20000.);
    }

    #[test]
    fn test_update_global() {
        let mut app = App::new();
        app.add_plugins((
            ClientPlugin::testing(BodiesConfig::SmallestBodyType(BodyType::Moon)),
            EnginePlugin,
        ));
        app.world
            .resource_mut::<NextState<ClientMode>>()
            .set(ClientMode::Explorer);
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
