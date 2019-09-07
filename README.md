# tcp-test - Test your TCP code

[![travis-badge]][travis]
[![appveyor-badge]][appveyor]
[![crates.io-badge]][crates.io]
[![docs-badge]][docs]
[![license-badge]][license]

[travis-badge]: https://travis-ci.com/Draphar/tcp-test.svg?branch=master
[travis]: https://travis-ci.com/Draphar/tcp-test
[appveyor-badge]: https://ci.appveyor.com/api/projects/status/github/Draphar/tcp-test?svg=true&branch=master
[appveyor]: https://ci.appveyor.com/project/Draphar/test-exec
[crates.io-badge]: https://img.shields.io/crates/v/tcp-test.svg
[crates.io]: https://crates.io/crates/tcp-test
[docs-badge]: https://docs.rs/tcp-test/badge.svg
[docs]: https://docs.rs/tcp-test
[license-badge]: https://img.shields.io/crates/l/tcp-test.svg
[license]: https://github.com/Draphar/tcp-test/blob/master/LICENSE

`tcp-test` is a Rust testing library to programmatically use real TCP in your tests.

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

    // both streams point to each other
    let local_addr = remote.local_addr().unwrap();
    let peer_addr = local.peer_addr().unwrap();
    assert_eq!(local_addr, peer_addr);

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
