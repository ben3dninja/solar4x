use bevy::prelude::*;
use ratatui::{
    buffer::Buffer,
    widgets::{Block, Borders, Paragraph, WidgetRef},
};

use crate::bodies::body_data::BodyData;



#[derive(Resource)]
pub struct InfoToggle(pub bool);

#[derive(Resource)]
pub struct InfoWidget {
    pub body_info: BodyData,
}

impl WidgetRef for InfoWidget {
    fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let body_info = &self.body_info;
        let info = Paragraph::new(format!(
            "Body type: {}\n\
            N of orbiting bodies: {}\n\
            Radius: {} km\n\
            Revolution period: {} earth days",
            body_info.body_type,
            body_info.orbiting_bodies.len(),
            body_info.radius,
            body_info.revolution_period,
        ))
        .block(
            Block::default()
                .title(&body_info.name[..])
                .borders(Borders::ALL),
        );
        info.render_ref(area, buf);
    }
}
