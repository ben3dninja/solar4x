use arrayvec::ArrayString;

pub const MAX_ID_LENGTH: usize = 32;

// #[derive(Default)]
// pub(super) struct NumberIncrementer(u64);

// impl NumberIncrementer {
//     fn next(&mut self) -> u64 {
//         let id = self.0;
//         self.0 += 1;
//         id
//     }
// }

// pub(super) trait IDBuilder {
//     type ID;

//     fn id_from_u64(u: u64) -> Self::ID;

//     fn incrementer(&mut self) -> &mut NumberIncrementer;

//     fn new_id(&mut self) -> Self::ID {
//         Self::id_from_u64(self.incrementer().next())
//     }
// }

pub fn id_from(s: &str) -> ArrayString<MAX_ID_LENGTH> {
    ArrayString::from(s).unwrap()
}
