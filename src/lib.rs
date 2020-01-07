mod prefix;
mod typed;

pub use prefix::{prefixed, prefixed_ro, PrefixedStorage, ReadonlyPrefixedStorage};
pub use typed::TypedStorage;
