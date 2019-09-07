use lazy_static::lazy_static;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tcp_test::{channel, read_assert};

lazy_static! {
    static ref DEFAULT_ADDRESS: SocketAddr =
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 31398));
}

macro_rules! send_read {
    ($data:expr) => {
        use std::io::Write;

        let (mut local, mut remote) = channel();

        assert_eq!(local.peer_addr().unwrap(), *DEFAULT_ADDRESS);
        assert_eq!(remote.local_addr().unwrap(), *DEFAULT_ADDRESS);

        local.write_all($data).unwrap();

        read_assert!(remote, 8, $data);
    };
}

#[test]
fn channel_0() {
    send_read!(b"channel0");
}

#[test]
fn channel_1() {
    send_read!(b"plA_1m&a");
}

#[test]
fn channel_2() {
    send_read!(b"Xn*>Yh68");
}

#[test]
fn channel_3() {
    send_read!(b"x?Ic98}R");
}

#[test]
fn channel_4() {
    send_read!(b"1_3JV-Qn");
}

#[test]
fn channel_5() {
    send_read!(b"2BAGFTG,");
}

#[test]
fn channel_6() {
    send_read!(b".OEk:={T");
}

#[test]
fn channel_7() {
    send_read!(b"Bw<73HwG");
}

#[test]
fn channel_8() {
    send_read!(b"EPbZ^XEH");
}

#[test]
fn channel_9() {
    send_read!(b"=Ml+{,*L");
}
