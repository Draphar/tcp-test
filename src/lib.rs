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

# Features

By default, a panic in one of the internal threads causes all tests to exit,
because in most cases the tests will just block indefinitely.
The `only_panic` feature prevents this behaviour if enabled.

[`channel()`]: fn.channel.html
*/

#![deny(unsafe_code)]

extern crate lazy_static;

use lazy_static::lazy_static;

use std::io::{self, Error, ErrorKind};
use std::mem;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream, ToSocketAddrs};
use std::process;
use std::sync::{Arc, Condvar, Mutex, Once};
use std::thread::Builder;

static SPAWN_SERVER: Once = Once::new();

lazy_static! {
    static ref STREAM: Arc<(Mutex<Option<(TcpStream, TcpStream)>>, Condvar)> =
        Arc::new((Mutex::new(None), Condvar::new()));

    /// `127.0.0.1:31398`
    static ref DEFAULT_ADDRESS: SocketAddr =
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 31398));
}

/// Listen to traffic on a specific address.
///
/// The parsed input address is returned for simplicity reasons.
///
/// # Important:
///
/// The address must be equal in all calls to this function,
/// otherwise only one of the addresses is used!
///
/// If there already is a server listening on *any* address,
/// only the address is returned even though it might not be the address of the listening server.
/// The same holds for `listen()`.
fn listen_on(address: impl ToSocketAddrs) {
    SPAWN_SERVER.call_once(move || {
        let address = address
            .to_socket_addrs()
            .expect("<impl ToSocketAddrs>::to_socket_addrs() at listen_on()")
            .next()
            .expect("ToSocketAddrs::Iter::next() at listen_on()");

        let listener = TcpListener::bind(address).expect("TcpListener::bind() at listen_on()");
        let buf = Arc::new((Mutex::new(None), Condvar::new()));
        let buf2 = buf.clone();

        Builder::new()
            .name(String::from("tcp-test listener thread"))
            .spawn(move || {
                let (ref lock, ref cvar) = &*buf2;

                listener_thread(listener, lock, cvar).map_err(|e| {
                    if cfg!(feature = "only_panic") {
                        panic!("tcp-test internal error: {}", e);
                    } else if cfg!(not(feature = "only_panic")) {
                        eprintln!(
                            "tcp-test internal error: {error}, {file}:{line}:{column}",
                            error = e,
                            file = file!(),
                            line = line!(),
                            column = column!()
                        );
                        process::exit(1);
                    };
                })
            })
            .expect("Builder::spawn() at listen_on()");

        Builder::new()
            .name(String::from("tcp-test channel thread"))
            .spawn(move || {
                let &(ref lock, ref cvar) = &*buf;

                channel_thread(address, lock, cvar).map_err(|e| {
                    if cfg!(feature = "only_panic") {
                        panic!("tcp-test internal error: {}", e);
                    } else if cfg!(not(feature = "only_panic")) {
                        eprintln!(
                            "tcp-test internal error: {error}, {file}:{line}:{column}",
                            error = e,
                            file = file!(),
                            line = line!(),
                            column = column!()
                        );
                        process::exit(1);
                    };
                })
            })
            .expect("Builder::spawn() at listen_on()");
    });
}

fn listener_thread(
    listener: TcpListener,
    lock: &Mutex<Option<TcpStream>>,
    cvar: &Condvar,
) -> io::Result<()> {
    let error = |message| Error::new(ErrorKind::Other, message);

    for i in listener.incoming() {
        let i = i?;

        let mut buf = lock
            .lock()
            .map_err(|_| error(concat!("failed to lock Mutex, ", line!())))?;

        while buf.is_some() {
            buf = cvar
                .wait(buf)
                .map_err(|_| error(concat!("failed to wait for Condvar, ", line!())))?;
        }

        *buf = Some(i);
        cvar.notify_one();
    }

    Ok(())
}

fn channel_thread(
    address: SocketAddr,
    lock: &Mutex<Option<TcpStream>>,
    cvar: &Condvar,
) -> Result<(), Error> {
    let error = |message| Error::new(ErrorKind::Other, message);

    loop {
        let local = TcpStream::connect(address)?;

        let remote = {
            // get the stream from the listener thread
            let mut remote = lock
                .lock()
                .map_err(|_| error(concat!("failed to lock Mutex, ", line!())))?;
            while remote.is_none() {
                remote = cvar
                    .wait(remote)
                    .map_err(|_| error(concat!("failed to wait for Condvar, ", line!())))?;
            }

            mem::replace(&mut *remote, None).unwrap()
        };

        // change the global variable
        let &(ref lock, ref cvar) = &*STREAM.clone();
        let mut stream = lock
            .lock()
            .map_err(|_| error(concat!("failed to lock Mutex, ", line!())))?;
        while stream.is_some() {
            stream = cvar
                .wait(stream)
                .map_err(|_| error(concat!("failed to wait for Condvar, ", line!())))?;
        }

        *stream = Some((local, remote));

        cvar.notify_one();
    }
}

/// Returns two TCP streams pointing at each other.
///
/// The internal TCP listener is bound to `127.0.0.1:31398`.
///
/// # Example
///
/// ```
/// # use tcp_test::channel;
/// use std::io::{Read, Write};
///
/// let data = b"Hello world!";
/// let (mut local, mut remote) = channel();
///
/// let local_addr = local.local_addr().unwrap();
/// let peer_addr = remote.peer_addr().unwrap();
///
/// assert_eq!(local_addr, peer_addr);
///
/// local.write_all(data).unwrap();
///
/// let mut buf = [0; 12];
/// remote.read_exact(&mut buf).unwrap();
///
/// assert_eq!(&buf, data);
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
/// # use tcp_test::channel_on;
/// use std::io::{Read, Write};
///
/// let data = b"Hello world!";
/// let (mut local, mut remote) = channel_on("127.0.0.1:31398");
///
/// let local_addr = remote.local_addr().unwrap();
/// let peer_addr = local.peer_addr().unwrap();
///
/// assert_eq!(local_addr, peer_addr);
///
/// local.write_all(data).unwrap();
///
/// let mut buf = [0; 12];
/// remote.read_exact(&mut buf).unwrap();
///
/// assert_eq!(&buf, data);
/// ```
///
/// [`listen_on()`]: fn.listen_on.html
pub fn channel_on(address: impl ToSocketAddrs) -> (TcpStream, TcpStream) {
    listen_on(address);

    let &(ref lock, ref cvar) = &*STREAM.clone();
    let mut buf = lock.lock().unwrap();
    while buf.is_none() {
        buf = cvar.wait(buf).unwrap();
    }

    let channel = mem::replace(&mut *buf, None);

    cvar.notify_all();

    channel.unwrap()
}

/// Convenience macro for reading and comparing a specific amount of bytes.
///
/// Reads a `$n` number of bytes from `$resource` and then compares that buffer with `$expected`.
/// Panics if the buffers are not equal.
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
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
