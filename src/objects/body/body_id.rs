use arrayvec::ArrayString;

use crate::MAX_ID_LENGTH;

pub type BodyID = ArrayString<MAX_ID_LENGTH>;

pub fn id_from(s: &str) -> BodyID {
    BodyID::from(s).unwrap()
}