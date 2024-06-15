use std::hash::{DefaultHasher, Hash, Hasher};

pub fn hash(t: &impl Hash) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
