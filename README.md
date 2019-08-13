# tcp-test - Test your TCP code

[![travis-badge]][travis]
[![crates.io-badge]][crates.io]
[![license-badge]][license]

[travis-badge]: https://img.shields.io/travis/com/Draphar/tcp-test.svg?branch=master&style=flat-square
[travis]: https://travis-ci.com/Draphar/tcp-test
[crates.io-badge]: https://img.shields.io/crates/v/tcp-test.svg?style=flat-square
[crates.io]: https://crates.io/crates/tcp-test
[license-badge]: https://img.shields.io/crates/l/tcp-test.svg?style=flat-square
[license]: https://github.com/Draphar/tcp-test/blob/master/LICENSE

`tcp-test` is a Rust testing library to programmatically use real TCP in your tests.

Warning: Windows is currently not supported because of `WSACancelBlockingCall` exceptions.

## Usage

`Cargo.toml`

```toml
[dev-dependencies]
tcp-test = "0.1"
```

Then simply use [`channel()`] in every test:

```rust
use tcp_test::channel;
use std::io::{self, Read, Write};

#[test]
fn some_test() {
    let (mut local, mut remote) = channel();
    
    let data = b"Hello, dear listener!";
    local.write_all(data).unwrap();
    
    let mut buf = Vec::new();
    remote.read_to_end(&mut buf).unwrap();
    
    assert_eq!(&buf, data);
}

#[test]
fn other_test() {
    let (mut local, mut remote) = channel();

    // ...
}
```

[`channel()`]: https://docs.rs/tcp-test/0.1.0/tcp_test/fn.channel.html