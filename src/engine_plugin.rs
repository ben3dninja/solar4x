use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
    utils::HashMap,
};

use crate::{
    app::body_id::BodyID,
    utils::algebra::{degs, mod_180, rads},
};

// Speed in days per second
const DEFAULT_SPEED: f64 = 10.;
pub struct Engine;

impl Plugin for Engine {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTime::default())
            .insert_resource(GameSpeed::default())
            .add_systems(FixedUpdate, (update_time, update_local, update_global));
    }
}

#[derive(Resource, Default)]
pub struct GameTime(pub f64);

#[derive(Resource)]
pub struct GameSpeed(pub f64);

impl Default for GameSpeed {
    fn default() -> Self {
        GameSpeed(DEFAULT_SPEED)
    }
}

fn update_time(mut game_time: ResMut<GameTime>, speed: Res<GameSpeed>, app_time: Res<Time>) {
    game_time.0 += speed.0 * app_time.delta_seconds_f64();
}

fn update_local(mut orbits: Query<&mut EllipticalOrbit>, time: Res<GameTime>) {
    orbits.par_iter_mut().for_each(|mut o| o.update_pos(time.0));
}

fn update_global(
    mut query: Query<(&mut Position, &EllipticalOrbit, &ParentBody)>,
    primary: Res<PrimaryBody>,
    mapping: Res<EntityMapping>,
) {
    let mut queue = vec![(primary.0, DVec3::ZERO)];
    let mut i = 0;
    while i < queue.len() {
        let (id, parent_pos) = queue[i];
        if let Some(entity) = mapping.id_mapping.get(&id) {
            if let Ok((mut world_pos, orbit, node)) = query.get_mut(*entity) {
                let pos = parent_pos + orbit.local_pos;
                world_pos.0 = pos;
                queue.extend(node.children.iter().map(|c| (*c, pos)));
            }
        }
        i += 1;
    }
}

#[derive(Resource)]
pub struct EntityMapping {
    id_mapping: HashMap<BodyID, Entity>,
}

#[derive(Resource)]
pub struct PrimaryBody(BodyID);

#[derive(Component)]
pub struct Position(DVec3);

#[derive(Component)]
pub struct ParentBody {
    children: Vec<BodyID>,
}

#[derive(Component)]
pub struct EllipticalOrbit {
    eccentricity: f64,
    semimajor_axis: f64,
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
