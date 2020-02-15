// derive macros
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm::errors::Result;
use cosmwasm::traits::Storage;

use crate::Singleton;

/// SeqVal holds a u64 sequence counter but wraps it as a newtype for clarity
/// but mainly to be able to derive NamedType.
/// If named_type included a default derivation for u64 and other primitives, we could
/// just use a u64 here.
#[derive(Serialize, Deserialize, Copy, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct SeqVal(pub u64);

/// Sequence creates a custom Singleton to hold an empty sequence
pub fn sequence<'a, S: Storage>(storage: &'a mut S, key: &[u8]) -> Singleton<'a, S, SeqVal> {
    Singleton::new(storage, key)
}

/// currval returns the last value returned by nextval. If the sequence has never been used,
/// then it will return 0.
pub fn currval<S: Storage>(seq: &Singleton<S, SeqVal>) -> Result<u64> {
    let val = seq.may_load()?;
    Ok(val.unwrap_or_default().0)
}

/// nextval increments the counter by 1 and returns the new value.
/// On the first time it is called (no sequence info in db) it will return 1.
pub fn nextval<S: Storage>(seq: &mut Singleton<S, SeqVal>) -> Result<u64> {
    let val = currval(&seq)? + 1;
    seq.save(&SeqVal(val))?;
    Ok(val)
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm::mock::MockStorage;

    #[test]
    fn walk_through_sequence() {
        let mut store = MockStorage::new();
        let mut seq = sequence(&mut store, b"seq");

        assert_eq!(currval(&seq).unwrap(), 0);
        assert_eq!(nextval(&mut seq).unwrap(), 1);
        assert_eq!(nextval(&mut seq).unwrap(), 2);
        assert_eq!(nextval(&mut seq).unwrap(), 3);
        assert_eq!(currval(&seq).unwrap(), 3);
        assert_eq!(currval(&seq).unwrap(), 3);
    }

    #[test]
    fn sequences_independent() {
        let mut store = MockStorage::new();

        let mut seq = sequence(&mut store, b"seq");
        assert_eq!(nextval(&mut seq).unwrap(), 1);
        assert_eq!(nextval(&mut seq).unwrap(), 2);
        assert_eq!(nextval(&mut seq).unwrap(), 3);

        let mut seq2 = sequence(&mut store, b"seq2");
        assert_eq!(nextval(&mut seq2).unwrap(), 1);
        assert_eq!(nextval(&mut seq2).unwrap(), 2);

        let mut seq3 = sequence(&mut store, b"seq");
        assert_eq!(nextval(&mut seq3).unwrap(), 4);
    }

    #[test]
    fn set_sequence() {
        let mut store = MockStorage::new();
        let mut seq = sequence(&mut store, b"seq");

        assert_eq!(nextval(&mut seq).unwrap(), 1);
        assert_eq!(nextval(&mut seq).unwrap(), 2);

        seq.save(&SeqVal(20)).unwrap();

        assert_eq!(currval(&seq).unwrap(), 20);
        assert_eq!(nextval(&mut seq).unwrap(), 21);
    }
}
