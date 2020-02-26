# CHANGELOG

## v0.2.0

* BREAKING: `Bucket.update()`` callback takes `Option<T>` not just `T`, allow it to work on unset values
* We no longer require `NamedType` derive statements or traits, just Rust 1.38+ stdlib
* Updated to CosmWasm 0.7.0

## v0.1.1

* Refactor structs to center around reusable functions, rather than copied methods
* Create Bucket to join PrefixedStorage and TypedStorage in one object

## v0.1.0

* Basic release with prefix stores, sequence, etc