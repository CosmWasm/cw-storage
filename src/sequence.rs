// derive macros
use named_type::NamedType;
use named_type_derive::NamedType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm::errors::Result;
use cosmwasm::traits::Storage;

use crate::Singleton;

/// SeqVal holds a u64 sequence counter but wraps it as a newtype for clarity
/// but mainly to be able to derive NamedType.
/// If named_type included a default derivation for u64 and other primitives, we could
/// just use a u64 here.
#[derive(Serialize, Deserialize, Copy, Clone, Default, Debug, PartialEq, JsonSchema, NamedType)]
pub struct SeqVal(pub u64);

/// Sequence creates a custom Singleton to hold an empty sequence
pub fn sequence<'a, S: Storage>(storage: &'a mut S, key: &[u8]) -> Singleton<'a, S, SeqVal> {
    Singleton::new(storage, key)
}

/// currval returns the last value returned by nextval. If the sequence has never been used,
/// then it will return 0.
pub fn currval<'a, S: Storage>(seq: &'a Singleton<'a, S, SeqVal>) -> Result<u64> {
    let val = seq.may_load()?;
    Ok(val.unwrap_or_default().0)
}

/// nextval increments the counter by 1 and returns the new value.
/// On the first time it is called (no sequence info in db) it will return 1.
pub fn nextval<'a, S: Storage>(seq: &'a mut Singleton<'a, S, SeqVal>) -> Result<u64> {
    let val = currval(&seq)? + 1;
    seq.save(&SeqVal(val))?;
    Ok(val)
}
