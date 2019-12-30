# cw-storage

[![CircleCI](https://circleci.com/gh/confio/cw-storage/tree/master.svg?style=shield)](https://circleci.com/gh/confio/cw-storage/tree/master) 

CosmWasm library with useful helpers for Storage patterns

## Contents

* [PrefixedStorage](#prefixed-storage)

### Prefixed Storage

One common technique in smart contracts, especially when multiple types of data
are being stored, is to create separate sub-stores with unique prefixes. Thus instead
of directly dealing with storage, we wrap it and put all `Foo` in a Storage with
key `"foo" + id`, and all `Bar` in a Storage with key `"bar" + id`. This lets us add multiple
types of objects without too much cognitive overhead. Similar separation like Mongo collections
or SQL tables.

Since we have different types for `Storage` and `ReadonlyStorage`, we use two different constructors:

```rust
use cw_storage::{prefixed, prefixed_ro};

let mut store = MockStorage::new();

let mut foos = prefixed(b"foo", &mut store);
foos.set(b"one", b"foo");

let mut bars = prefixed(b"bar", &mut store);
bars.set(b"one", b"bar");

let read_foo = prefixed_ro(b"foo", &store);
assert_eq!(b"foo".to_vec(), read_foo.get(b"one").unwrap());

let read_bar = prefixed_ro(b"bar", &store);
assert_eq!(b"bar".to_vec(), read_bar.get(b"one").unwrap());
```

Please note that only one mutable reference to the underlying store may be valid at one point.
The compiler sees we do not ever use `foos` after constructing `bars`, so this example is valid.
However, if we did use `foos` again at the bottom, it would properly complain about violating
unique mutable reference. 

The takeaway is to create the `PrefixedStorage` objects when needed and not to hang around to them too long.
