#![allow(dead_code)]
// remote this when we use index
use serde::{Deserialize, Serialize};

use cosmwasm::errors::Result;
use cosmwasm::traits::{ReadonlyStorage, Storage};

use crate::namespace_helpers::key_prefix;
use crate::typed::{typed, typed_read};

pub fn index<T, F>(namespace: &[u8], action: F) -> Index<T>
where
    F: Fn(&T) -> Vec<u8> + 'static,
{
    Index {
        prefix: key_prefix(namespace),
        action: Box::new(action),
    }
}

// TODO: add unique field
pub struct Index<T> {
    prefix: Vec<u8>,
    action: Box<dyn Fn(&T) -> Vec<u8>>,
}

impl<T> Index<T> {
    fn calc_key(&self, item: &T) -> Vec<u8> {
        let calc = (self.action)(item);
        let mut k = self.prefix.clone();
        k.extend_from_slice(&calc);
        k
    }
}

// TODO: make this Base64 in 0.7.0
type Ref = Vec<u8>;

/// IndexEntry is persisted to disk and lists all primary keys that have a given index value
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
struct IndexEntry {
    pub refs: Vec<Ref>,
}

/*
This is getting expensive.
Saving an item without index is 1 write
Creating an item with 1 index is 2 read + 2 write (1 read to check old value, 1 read+write to add_ref)
Updating an item with 1 index is 3 read + 3 write (1 read to check old value, 1 read+write to add_ref, 1 read+write to remove_ref)

It *may* be possible to reduce the number of reads, but writes cannot change
*/

// must do a read for old data
pub fn write_index<S: Storage, T>(
    storage: &mut S,
    idx: &Index<T>,
    pk: &[u8],
    old_val: Option<&T>,
    new_val: &T,
) -> Result<()> {
    let old_idx = old_val.map(|o| idx.calc_key(o));
    let new_idx = idx.calc_key(new_val);

    // no change is a no-op
    if let Some(o) = &old_idx {
        // if it unchanged, it is a no-op
        if o == &new_idx {
            return Ok(());
        }
        // otherwise, remove it
        remove_ref(storage, o.as_slice(), pk)?;
    }

    // now add the new pk
    add_ref(storage, new_idx.as_slice(), pk)
}

/// read_refs find all references that match the template object
/// this can just have the 1 field set that the Index needs to calculate the key from
pub fn read_refs<S: ReadonlyStorage, T>(
    storage: &S,
    idx: &Index<T>,
    template: &T,
) -> Result<Vec<Ref>> {
    let idx_key = idx.calc_key(template);
    let entry = load_refs(storage, &idx_key)?.unwrap_or_default();
    Ok(entry.refs)
}

pub fn remove_ref<S: Storage>(storage: &mut S, idx: &[u8], pk: &[u8]) -> Result<()> {
    let mut db = typed(storage);
    let mut entry: IndexEntry = db.load(idx)?;
    // TODO: error if not found?
    entry.refs = entry
        .refs
        .into_iter()
        .filter(|r| r.as_slice() != pk)
        .collect();
    db.save(idx, &entry)
}

pub fn add_ref<S: Storage>(storage: &mut S, idx: &[u8], pk: &[u8]) -> Result<()> {
    let mut db = typed(storage);
    let mut entry: IndexEntry = db.may_load(idx)?.unwrap_or_default();
    // TODO: sort them?
    entry.refs.push(pk.to_vec());
    db.save(idx, &entry)
}

fn load_refs<S: ReadonlyStorage>(storage: &S, idx: &[u8]) -> Result<Option<IndexEntry>> {
    let db = typed_read(storage);
    db.may_load(idx)
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm::mock::MockStorage;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
    struct Person {
        pub name: String,
        pub age: u32,
    }

    #[test]
    fn build_index() {
        let idx = index(b"foo", |p: &Person| p.age.to_be_bytes().to_vec());

        let expected = vec![0u8, 3, b'f', b'o', b'o', 0, 0, 0, 127];
        let trial = idx.calc_key(&Person {
            name: "Fred".to_string(),
            age: 127,
        });
        assert_eq!(trial, expected);
    }

    #[test]
    fn add_refs_works() {
        let mut store = MockStorage::new();

        let pk: &[u8] = b"primary";
        let pk2: &[u8] = b"second";
        let idx: &[u8] = b"special key";

        let loaded = load_refs(&store, idx).unwrap();
        assert_eq!(loaded, None);

        add_ref(&mut store, idx, pk).unwrap();
        let loaded = load_refs(&store, idx).unwrap().unwrap();
        assert_eq!(loaded.refs, vec![pk]);

        add_ref(&mut store, idx, pk2).unwrap();
        let loaded = load_refs(&store, idx).unwrap().unwrap();
        assert_eq!(loaded.refs, vec![pk, pk2]);

        // TODO: test adding same ref second time -> ensure not added
    }

    #[test]
    fn remove_refs_works() {
        let mut store = MockStorage::new();

        let pk: &[u8] = b"primary";
        let pk2: &[u8] = b"second";
        let idx: &[u8] = b"special key";

        // set up with 2
        add_ref(&mut store, idx, pk).unwrap();
        add_ref(&mut store, idx, pk2).unwrap();

        // remove one and see change
        remove_ref(&mut store, idx, pk).unwrap();
        let loaded = load_refs(&store, idx).unwrap().unwrap();
        assert_eq!(loaded.refs, vec![pk2]);

        // ignore second removal (TODO: is this right?)
        remove_ref(&mut store, idx, pk).unwrap();
        let loaded = load_refs(&store, idx).unwrap().unwrap();
        assert_eq!(loaded.refs, vec![pk2]);

        // goes to empty (not None)
        remove_ref(&mut store, idx, pk2).unwrap();
        let loaded = load_refs(&store, idx).unwrap().unwrap();
        assert_eq!(loaded.refs.len(), 0);

        // can add again later
        add_ref(&mut store, idx, pk2).unwrap();
        let loaded = load_refs(&store, idx).unwrap().unwrap();
        assert_eq!(loaded.refs, vec![pk2]);
    }

    #[test]
    fn create_with_index() {
        let mut store = MockStorage::new();

        let idx = index(b"foo", |p: &Person| p.age.to_be_bytes().to_vec());
        let bob = Person {
            name: "Roberto".to_string(),
            age: 66,
        };

        write_index(&mut store, &idx, b"where", None, &bob).unwrap();

        // make sure it is there
        let mut template = Person::default();
        template.age = 66;
        let refs = read_refs(&store, &idx, &template).unwrap();
        assert_eq!(refs, vec![b"where".to_vec()]);

        // but not for another age
        template.age = 67;
        let refs = read_refs(&store, &idx, &template).unwrap();
        assert_eq!(refs, Vec::<Ref>::new());
    }

    #[test]
    fn update_with_index() {
        let mut store = MockStorage::new();

        let idx = index(b"foo", |p: &Person| p.age.to_be_bytes().to_vec());
        let bob = Person {
            name: "Roberto".to_string(),
            age: 66,
        };

        // set initial age
        write_index(&mut store, &idx, b"where", None, &bob).unwrap();

        // time for a birthday
        let mut bobby = bob.clone();
        bobby.age += 1;
        write_index(&mut store, &idx, b"where", Some(&bob), &bobby).unwrap();

        // make sure it is at the new one
        let mut template = Person::default();
        template.age = 67;
        let refs = read_refs(&store, &idx, &template).unwrap();
        assert_eq!(refs, vec![b"where".to_vec()]);

        // and gone from the old one
        template.age = 66;
        let refs = read_refs(&store, &idx, &template).unwrap();
        assert_eq!(refs, Vec::<Ref>::new());
    }
}
