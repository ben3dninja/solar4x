use std::collections::HashMap;

use super::{body_data::BodyData, body_id::BodyID};

pub struct SystemInfo {
    pub bodies: HashMap<BodyID, BodyData>,
    pub primary_body: BodyID,
}

impl SystemInfo {
    pub fn new<T: IntoIterator<Item = BodyData>>(bodies: T) -> Option<Self> {
        let collect = bodies.into_iter().map(|data| (data.id, data)).collect();
        let bodies: HashMap<BodyID, BodyData> = collect;

        bodies
            .values()
            .find(|data| data.host_body.is_none())
            .map(|data| data.id)
            .map(|primary_body| SystemInfo {
                bodies,
                primary_body,
            })
    }
    pub fn get_max_distance(&self) -> i64 {
        self.bodies
            .values()
            .map(|body| body.semimajor_axis)
            .max()
            .unwrap()
    }

    pub fn get_body_ancestors(&self, mut id: BodyID) -> Vec<BodyID> {
        let bodies = &self.bodies;
        let mut ancestors = Vec::new();
        let mut body_option = bodies.get(&id);
        while let Some(body) = body_option {
            if body.host_body.is_none() {
                break;
            }
            id = body.host_body.unwrap();
            body_option = self.bodies.get(&id);
            ancestors.push(id);
        }
        ancestors.reverse();
        ancestors
    }
}

#[cfg(test)]
mod tests {
    use crate::{app::body_data::BodyType, standalone::Standalone};

    #[test]
    fn test_max_distance() {
        let (app, _) = Standalone::new_testing(BodyType::Planet).unwrap();
        assert_eq!(app.core().shared_info.get_max_distance(), 4498396441)
    }

    #[test]
    fn test_primary_body() {
        let (app, _) = Standalone::new_testing(BodyType::Planet).unwrap();
        assert_eq!(app.core().shared_info.primary_body, "soleil".into())
    }

    #[test]
    fn test_get_body_ancestors() {
        let (app, _) = Standalone::new_testing(BodyType::Moon).unwrap();
        assert_eq!(
            app.core().shared_info.get_body_ancestors("lune".into()),
            vec!["soleil".into(), "terre".into()]
        );
    }
}
