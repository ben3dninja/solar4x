use arrayvec::ArrayString;
use bevy::{math::DVec3, prelude::*, utils::HashMap};

use super::id::{IDBuilder, NumberIncrementer};

pub(crate) struct ShipID(u64);


#[derive(Resource, Default)]
struct ShipIDBuilder(NumberIncrementer);

impl IDBuilder for ShipIDBuilder {
    type ID=ShipID;

    fn incrementer(&mut self) -> &mut NumberIncrementer {
        &mut self.0
    }
    
    fn id_from_u64(u: u64) -> Self::ID {
        ShipID(u)
    }
}

#[derive(Component, Clone, Default, PartialEq)]
pub struct ShipInfo {
    pub id: ShipID,
    pub spawn_pos: DVec3,
    pub spawn_speed: DVec3,
}

#[derive(Resource, Default)]
pub struct ShipsMapping(pub HashMap<ShipID, Entity>);

#[derive(Event)]
pub enum ShipEvent {
    Create(ShipInfo),
    Remove(ShipID),
}

fn handle_ship_events(
    mut commands: Commands,
    mut reader: EventReader<ShipEvent>,
    mut ships: ResMut<ShipsMapping>,
    bodies: Query<(&Position, &HillRadius, &BodyInfo)>,
    mapping: Res<BodiesMapping>,
    main_body: Query<&BodyInfo, With<PrimaryBody>>,
) {
    for event in reader.read() {
        match event {
            ShipEvent::Create(info) => {
                let pos = Position(info.spawn_pos);
                ships.0.entry(info.id).or_insert({
                    let influence =
                        compute_influence(&pos, &bodies, mapping.as_ref(), main_body.single().0.id);
                    commands
                        .spawn((
                            info.clone(),
                            Acceleration::new(get_acceleration(
                                info.spawn_pos,
                                bodies
                                    .iter_many(&influence.influencers)
                                    .map(|(p, _, i)| (p.0, i.0.mass)),
                            )),
                            influence,
                            pos,
                            Velocity(info.spawn_speed),
                            TransformBundle::from_transform(Transform::from_xyz(0., 0., 1.)),
                        ))
                        .id()
                });
            }
            ShipEvent::Remove(id) => {
                if let Some(e) = ships.0.remove(id) {
                    commands.entity(e).despawn()
                }
            }
        }
    }
}