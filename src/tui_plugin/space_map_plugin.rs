use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
    utils::HashMap,
};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Stylize},
    widgets::{
        block::Title,
        canvas::{Canvas, Circle},
        Block, WidgetRef,
    },
};

use crate::{
    bodies::{
        body_data::{BodyData, BodyType},
        body_id::BodyID,
    },
    core_plugin::BodyInfo,
    engine_plugin::{Position, ToggleTime},
    utils::{
        algebra::project_onto_plane,
        ui::{Direction2, Direction4},
    },
};

use super::AppScreen;

const OFFSET_STEP: f64 = 1e8;
const ZOOM_STEP: f64 = 1.5;
pub struct SpaceMapPlugin;

impl Plugin for SpaceMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            // No need to ask it to run only in explorer screen as the check is done inside the function to acquire the context
            update_space_map.run_if(resource_equals(ToggleTime(true))),
        );
    }
}

#[derive(Debug)]
pub enum SpaceMapEvent {
    Zoom(Direction2),
    MapOffset(Direction4),
    MapOffsetReset,
    FocusBody,
    Autoscale,
}

#[derive(Default, Debug)]
pub struct SpaceMap {
    circles: Vec<Circle>,
    pub offset_amount: DVec2,
    pub zoom_level: f64,
    pub system_size: f64,
}

impl WidgetRef for SpaceMap {
    fn render_ref(&self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let (width, height) = (area.width as f64, area.height as f64);
        let scale = self.system_size / (width.min(height) * self.zoom_level);
        let (width, height) = (width * scale, height * scale);
        Canvas::default()
            .block(
                Block::bordered()
                    .title(Title::from("Space map".bold()).alignment(Alignment::Center)),
            )
            .x_bounds([-width / 2., width / 2.])
            .y_bounds([-height, height])
            .paint(|ctx| {
                for circle in &self.circles {
                    ctx.draw(circle);
                }
            })
            .render_ref(area, buf)
    }
}

fn update_space_map(query: Query<(&Position, &BodyInfo)>, mut screen: ResMut<AppScreen>) {
    if let AppScreen::Explorer(ctx) = screen.as_mut() {
        let mut circles = Vec::new();
        let &Position(focus_pos) = query.get(ctx.focus_body).unwrap().0;
        let selected = ctx.tree_state.selected_body_id();
        for (&Position(pos), BodyInfo(data)) in query.iter() {
            let proj = project_onto_plane(pos - focus_pos, (DVec3::X, DVec3::Y))
                - ctx.space_map.offset_amount;
            let color = match data.body_type {
                _ if data.id == selected => Color::Red,
                BodyType::Star => Color::Yellow,
                BodyType::Planet => Color::Blue,
                _ => Color::DarkGray,
            };
            let radius = data.radius;
            circles.push(Circle {
                x: proj.x,
                y: proj.y,
                radius,
                color,
            });
        }
        ctx.space_map.circles = circles;
    }
}

impl SpaceMap {
    pub fn new<'a>(body_positions: impl Iterator<Item = &'a Position>) -> SpaceMap {
        let system_size = body_positions
            .map(|pos| pos.0.length())
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        SpaceMap {
            circles: Vec::new(),
            offset_amount: DVec2::ZERO,
            zoom_level: 1.,
            system_size,
        }
    }

    pub fn zoom_in(&mut self) {
        self.zoom_level *= ZOOM_STEP;
    }

    pub fn zoom_out(&mut self) {
        self.zoom_level /= ZOOM_STEP;
    }

    pub fn zoom(&mut self, direction: Direction2) {
        match direction {
            Direction2::Up => self.zoom_in(),
            Direction2::Down => self.zoom_out(),
        }
    }

    pub fn offset(&mut self, direction: Direction4) {
        use Direction4::*;
        self.offset_amount += (match direction {
            Front | Right => 1.,
            _ => -1.,
        } * OFFSET_STEP
            / self.zoom_level)
            * match direction {
                Front | Back => DVec2::Y,
                _ => DVec2::X,
            }
    }

    pub fn reset_offset(&mut self) {
        self.offset_amount = DVec2::ZERO;
    }

    pub fn autoscale(
        &mut self,
        focus_data: &BodyData,
        id_mapping: &HashMap<BodyID, Entity>,
        bodies: &Query<&BodyInfo>,
    ) {
        if let Some(max_dist) = focus_data
            .orbiting_bodies
            .iter()
            .filter_map(|id| {
                id_mapping
                    .get(id)
                    .and_then(|&e| bodies.get(e).ok())
                    .map(|body| body.0.semimajor_axis)
            })
            .max_by(|a, b| a.total_cmp(b))
        {
            self.zoom_level = self.system_size / max_dist;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::{
        app::{App, Update},
        ecs::schedule::IntoSystemConfigs,
    };

    use crate::{
        client_plugin::{ClientMode, ClientPlugin},
        core_plugin::{BodiesConfig, BodyInfo, SimulationSet},
        engine_plugin::{update_global, update_local, update_time, EnginePlugin},
        tui_plugin::{
            explorer_screen::ExplorerEvent,
            space_map_plugin::{update_space_map, SpaceMapEvent},
            AppScreen, ChangeAppScreen, TuiPlugin,
        },
    };

    #[test]
    fn test_update_space_map() {
        let mut app = App::new();
        app.add_plugins((
            ClientPlugin::testing(BodiesConfig::default(), ClientMode::Explorer),
            EnginePlugin,
            TuiPlugin::testing(Some(ChangeAppScreen::Explorer)),
        ))
        .add_systems(
            Update,
            (update_time, update_local, update_global, update_space_map)
                .in_set(SimulationSet)
                .chain(),
        );
        app.update();
        app.update();
        if let AppScreen::Explorer(ctx) = app.world.resource::<AppScreen>() {
            let map = &ctx.space_map;
            assert_eq!(map.circles.len(), 9);
            dbg!(map);
            assert!(4459753056. < map.system_size);
            assert!(map.system_size < 4537039826.);
        }
    }

    #[test]
    fn test_change_focus_body() {
        let mut app = App::new();
        app.add_plugins((
            ClientPlugin::testing(BodiesConfig::default(), ClientMode::Explorer),
            EnginePlugin,
            TuiPlugin::testing(Some(ChangeAppScreen::Explorer)),
        ));
        app.update();
        app.update();
        let earth = "terre".into();
        if let AppScreen::Explorer(ctx) = app.world.resource_mut::<AppScreen>().as_mut() {
            ctx.tree_state.select_body(earth);
        }
        app.update();

        app.world
            .send_event(ExplorerEvent::SpaceMap(SpaceMapEvent::FocusBody));
        app.update();
        if let AppScreen::Explorer(ctx) = app.world.resource_mut::<AppScreen>().as_mut() {
            let focus = ctx.focus_body;
            let world = &mut app.world;
            println!("{focus:?}");

            assert_eq!(
                world.query::<&BodyInfo>().get(world, focus).unwrap().0.id,
                earth
            );
        }
    }
}
