# cw-storage

[![CircleCI](https://circleci.com/gh/confio/cw-storage/tree/master.svg?style=shield)](https://circleci.com/gh/confio/cw-storage/tree/master) 

CosmWasm library with useful helpers for Storage patterns.
This is not in the core library, so feel free to fork it and modify or extend as desired for your contracts.
Pull Requests back to upstream repo with new or improved features are always welcome.

## Contents

* [PrefixedStorage](#prefixed-storage)
* [TypedStoreage](#typed-storage)

### Prefixed Storage

One common technique in smart contracts, especially when multiple types of data
are being stored, is to create separate sub-stores with unique prefixes. Thus instead
of directly dealing with storage, we wrap it and put all `Foo` in a Storage with
key `"foo" + id`, and all `Bar` in a Storage with key `"bar" + id`. This lets us add multiple
types of objects without too much cognitive overhead. Similar separation like Mongo collections
or SQL tables.

Since we have different types for `Storage` and `ReadonlyStorage`, we use two different constructors:

```rust
use cw_storage::{prefixed, prefixed_read};

let mut store = MockStorage::new();

let mut foos = prefixed(b"foo", &mut store);
foos.set(b"one", b"foo");

let mut bars = prefixed(b"bar", &mut store);
bars.set(b"one", b"bar");

let read_foo = prefixed_read(b"foo", &store);
assert_eq!(b"foo".to_vec(), read_foo.get(b"one").unwrap());

let read_bar = prefixed_read(b"bar", &store);
assert_eq!(b"bar".to_vec(), read_bar.get(b"one").unwrap());
```

Please note that only one mutable reference to the underlying store may be valid at one point.
The compiler sees we do not ever use `foos` after constructing `bars`, so this example is valid.
However, if we did use `foos` again at the bottom, it would properly complain about violating
unique mutable reference. 

The takeaway is to create the `PrefixedStorage` objects when needed and not to hang around to them too long.

### Typed Storage

As we divide our storage space into different subspaces or "buckets", we will quickly notice that each
"bucket" works on a unique type. This leads to a lot of repeated serialization and deserialization
boilerplate that can be removed. We do this by wrapping a `Storage` with a type-aware `TypedStorage`
struct that provides us a higher-level access to the data. 

Note that `TypedStorage` itself does not implement the `Storage` interface, so when combining 
with `PrefixStorage`, make sure to wrap the prefix first.

```rust
use cosmwasm::mock::MockStorage;
use cw_storage::{prefixed, typed};

let mut store = MockStorage::new();
let mut space = prefixed(b"data", &mut store);
let mut bucket = typed::<_, Data>(&mut space);

// save data
let data = Data {
    name: "Maria".to_string(),
    age: 42,
};
bucket.save(b"maria", &data).unwrap();

// load it properly
let loaded = bucket.load(b"maria").unwrap();
assert_eq!(data, loaded);

// loading empty can return Ok(None) or Err depending on the chosen method:
assert!(bucket.load(b"john").is_err());
assert_eq!(bucket.may_load(b"john"), Ok(None));
```

Beyond the basic `save`, `load`, and `may_load`, there is a higher-level API exposed, `update`.
`Update` will load the data, apply an operation and save it again (if the operation was successful).
It will also return any error that occurred, or the final state that was written if successful.

```rust
let birthday = |mut d: Data| {
    d.age += 1;
    Ok(d)
};
let output = bucket.update(b"maria", &birthday).unwrap();
let expected = Data {
    name: "Maria".to_string(),
    age: 43,
};
assert_eq!(output, expected);
``` 

Since the heart of much of the smart contract code is simply transformations upon some stored state,
We may be able to just code the transforms and let the `TypedStorage` APIs take care of all the boilerplate.
