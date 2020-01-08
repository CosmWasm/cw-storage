mod prefix;
mod singleton;
mod typed;

pub use prefix::{prefixed, prefixed_read, PrefixedStorage, ReadonlyPrefixedStorage};
pub use typed::{typed, typed_read, ReadonlyTypedStorage, TypedStorage};
