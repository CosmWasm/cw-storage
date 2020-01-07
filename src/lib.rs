mod prefix;
mod typed;

pub use prefix::{prefixed, prefixed_ro, PrefixedStorage, ReadonlyPrefixedStorage};
pub use typed::{typed, typed_read, ReadonlyTypedStorage, TypedStorage};
