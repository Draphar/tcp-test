use tcp_test::{channel, read_assert};

macro_rules! send_read {
    ($data:expr, $n:expr) => {
        // $n until [T; N].len() is a stable const fn
        use std::io::Write;

        let (mut local, mut remote) = channel();

        local.write_all($data).unwrap();

        read_assert!(remote, $n, $data);
    };
}

#[test]
fn channel_0() {
    send_read!(b"channel_0", 9);
}

#[test]
fn channel_1() {
    send_read!(b"plA_1m&a", 8);
}

#[test]
fn channel_2() {
    send_read!(b"Xn*>Yh68", 8);
}

#[test]
fn channel_3() {
    send_read!(b"x?Ic98}R", 8);
}

#[test]
fn channel_4() {
    send_read!(b"1_3JV-Qn", 8);
}

#[test]
fn channel_5() {
    send_read!(b"2BAGFTG,", 8);
}

#[test]
fn channel_6() {
    send_read!(b".OEk:={T", 8);
}

#[test]
fn channel_7() {
    send_read!(b"Bw<73HwG", 8);
}

#[test]
fn channel_8() {
    send_read!(b"EPbZ^XEH", 8);
}

#[test]
fn channel_9() {
    send_read!(b"=Ml+{,*L", 8);
}
