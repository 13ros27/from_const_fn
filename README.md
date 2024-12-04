[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/13ros27/from_const_fn#license)
[![CI](https://github.com/bevyengine/bevy/workflows/CI/badge.svg)](https://github.com/13ros27/from_const_fn/actions)

A `const` counterpart to `core::array::from_fn`, `from_const_fn!`.

This works similarly to [`from_fn`](https://doc.rust-lang.org/std/array/fn.from_fn.html) however also works in any `const` environment. This does come with the limitation of not supporting closures that borrow (or move) their environment. For more details and examples, see [`from_const_fn`](https://github.com/13ros27/from_const_fn/blob/master/lib.rs#L32).

## License

All code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
