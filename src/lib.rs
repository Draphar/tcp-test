/*!
Programmatically test TCP programs using real TCP streams.

# Example

Everything can be done using the [`channel()`] function:

```
use tcp_test::{channel, read_assert};
use std::io::{Read, Write};

#[test]
fn first_test() {
    let sent = b"Hello, reader";

    let (mut reader, mut writer) = channel();

    writer.write_all(sent).unwrap();

    let mut read = Vec::new();
    reader.read_to_end(&mut read).unwrap();

    assert_eq!(read, sent);
}

#[test]
fn second_test() {
    let sent = b"Interesting story";

    let (mut reader, mut writer) = channel();

    writer.write_all(sent).unwrap();

    read_assert!(reader, sent.len(), sent);
}

#[test]
fn third_test() {
    let sent = b"...";

    let (mut reader, mut writer) = channel();

    writer.write_all(sent).unwrap();

    read_assert!(reader, sent.len(), sent);
}
```

[`channel()`]: fn.channel.html
*/

//todo: Don't use ToSocketAddrs, create a new trait instead

extern crate lazy_static;

use lazy_static::lazy_static;

use std::net::*;
use std::sync::{mpsc, Arc, Mutex, Once};
use std::thread::Builder;

lazy_static! {
    /// `127.0.0.1:31398`
    static ref DEFAULT_ADDRESS: SocketAddr =
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 31398));
}

static mut CHANNEL: Option<Arc<Mutex<(mpsc::Sender<()>, mpsc::Receiver<(TcpStream, TcpStream)>)>>> =
    None;
static INIT: Once = Once::new();

fn init(address: impl ToSocketAddrs) {
    INIT.call_once(move || {
        let address = resolve(address);

        // channel for blocking
        let (ex_send, receiver) = mpsc::channel();

        // channel for sending the streams
        let (sender, ex_recv) = mpsc::channel();

        unsafe {
            CHANNEL = Some(Arc::new(Mutex::new((ex_send, ex_recv))));
        };

        let listener = TcpListener::bind(address)
            .expect(concat!("TcpListener::bind() at init(), line ", line!()));

        Builder::new()
            .name(String::from("tcp-test background thread"))
            .spawn(move || loop {
                receiver
                    .recv()
                    .expect(concat!("Receiver::recv() at init(), line ", line!()));

                let local = TcpStream::connect(address)
                    .expect(concat!("TcpStream::connect() at init(), line ", line!()));
                let remote = listener
                    .accept()
                    .expect(concat!("TcpListener::accept() at init(), line ", line!()))
                    .0;

                sender
                    .send((local, remote))
                    .expect(concat!("Sender::send() at init(), line ", line!()));
            })
            .expect(concat!("Builder::spawn() at init(), line ", line!()));
    });
}

/// Returns two TCP streams pointing at each other.
///
/// The internal TCP listener is bound to `127.0.0.1:31398`.
///
/// # Example
///
/// ```
/// use tcp_test::channel;
/// use std::io::{Read, Write};
///
/// #[test]
/// fn test() {
///     let data = b"Hello world!";
///     let (mut local, mut remote) = channel();
///
///     let local_addr = local.local_addr().unwrap();
///     let peer_addr = remote.peer_addr().unwrap();
///
///     assert_eq!(local_addr, peer_addr);
///     assert_eq!(local.peer_addr().unwrap(), "127.0.0.1:31398".parse().unwrap()); // default address
///
///     local.write_all(data).unwrap();
///
///     let mut buf = [0; 12];
///     remote.read_exact(&mut buf).unwrap();
///
///     assert_eq!(&buf, data);
/// }
/// ```
///
/// Also see the [module level example](index.html#example).
///
/// [`listen()`]: fn.listen.html
#[inline]
pub fn channel() -> (TcpStream, TcpStream) {
    channel_on(*DEFAULT_ADDRESS)
}

/// Returns two TCP streams pointing at each other.
///
/// The internal TCP listener is bound to `address`.
/// Only one listener is used throughout the entire program,
/// so the address should match in all calls to this function,
/// otherwise it is not specified which address is finally used.
///
/// # Example
///
/// ```
/// use tcp_test::channel_on;
/// use std::io::{Read, Write};
///
/// #[test]
/// fn test() {
///     let data = b"Hello world!";
///     let (mut local, mut remote) = channel_on("127.0.0.1:31399");
///
///     assert_eq!(local.peer_addr().unwrap(), "127.0.0.1:31399".parse().unwrap());
///     assert_eq!(remote.local_addr().unwrap(), "127.0.0.1:31399".parse().unwrap());
///
///     local.write_all(data).unwrap();
///
///     let mut buf = [0; 12];
///     remote.read_exact(&mut buf).unwrap();
///
///     assert_eq!(&buf, data);
/// }
/// ```
///
/// [`listen_on()`]: fn.listen_on.html
#[inline]
pub fn channel_on(address: impl ToSocketAddrs) -> (TcpStream, TcpStream) {
    init(address);

    let lock = unsafe { CHANNEL.clone().unwrap() };

    let guard = lock
        .lock()
        .expect(concat!("Mutex::lock() at channel_on(), line ", line!()));

    guard
        .0
        .send(())
        .expect(concat!("Sender::send() at channel_on(), line ", line!()));

    guard
        .1
        .recv()
        .expect(concat!("Receiver::recv() at channel_on(), line ", line!()))
}

/// Get the first socket address.
#[inline]
fn resolve(address: impl ToSocketAddrs) -> SocketAddr {
    address
        .to_socket_addrs()
        .expect(concat!(
            "<impl ToSocketAddrs>::to_socket_addrs() at resolve(), line ",
            line!()
        ))
        .next()
        .expect(concat!(
            "ToSocketAddrs::Iter::next() at resolve(), line ",
            line!()
        ))
}

/// Convenience macro for reading and comparing a specific amount of bytes.
///
/// Reads a `$n` number of bytes from `$resource` and then compares that buffer with `$expected`.
/// Panics if the buffers are not equal.
///
/// # Example
///
/// ```
/// use tcp_test::read_assert;
/// use std::io::{self, Read};
///
/// struct Placeholder;
///
/// impl Read for Placeholder {
///     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
///         buf[0] = 1;
///         buf[1] = 2;
///         buf[2] = 3;
///
///         Ok(3)
///     }
/// }
///
/// read_assert!(Placeholder {}, 3, [1, 2, 3]);
/// ```
#[macro_export]
macro_rules! read_assert {
    ($resource:expr, $n:expr, $expected:expr) => {{
        match &$expected {
            expected => {
                use std::io::Read;

                let mut buf = [0; $n];
                $resource
                    .read_exact(&mut buf)
                    .expect("failed to read in read_assert!");

                assert_eq!(
                    &buf[..],
                    &expected[..],
                    "read_assert! buffers are not equal"
                );
            }
        };
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_ok() {
        assert_eq!(resolve("127.0.0.1:80"), "127.0.0.1:80".parse().unwrap());
        assert_eq!(resolve("[::1]:80"), "[::1]:80".parse().unwrap());

        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80));
        let addrs = [addr; 3];
        assert_eq!(resolve(addrs.as_ref()), addr);
    }

    #[test]
    #[should_panic]
    fn resolve_err() {
        let addrs: [SocketAddr; 0] = [];
        resolve(addrs.as_ref());
    }

    use std::io::{self, Read};

    struct Placeholder;

    impl Read for Placeholder {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
    }

    #[test]
    fn read_assert_ok() {
        read_assert!(Placeholder {}, 9, [0; 9]);
    }

    #[test]
    #[should_panic]
    fn read_assert_panic() {
        read_assert!(Placeholder {}, 1, [0xff]);
    }

    macro_rules! test {
        () => {
            let (local, remote) = channel();

            let local_addr = remote.local_addr().unwrap();
            let peer_addr = local.peer_addr().unwrap();
            assert_eq!(local_addr, peer_addr);
        };
    }

    #[test]
    fn channel_0() {
        test!();
    }

    #[test]
    fn channel_1() {
        test!();
    }

    #[test]
    fn channel_2() {
        test!();
    }

    #[test]
    fn channel_3() {
        test!();
    }

    #[test]
    fn channel_4() {
        test!();
    }
}
