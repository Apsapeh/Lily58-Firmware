use crate::fixed_vec::FixedVec;
use shared_src::PrimitiveBitset;
use usbd_human_interface_device::{
    device::consumer::MultipleConsumerReport,
    page::{Consumer, Keyboard},
};

#[derive(Copy, Clone)]
enum MultiKey {
    ConsumerKey(Consumer),
    KeyboardKey(Keyboard),
}

type KeybardMatrixLayout = [MultiKey; 30];

struct KeyboardLayout {
    left: [KeybardMatrixLayout; 3],
    right: [KeybardMatrixLayout; 3],
}

macro_rules! key {
    ($key: ident) => {
        MultiKey::KeyboardKey(Keyboard::$key)
    };
}

macro_rules! consumer {
    ($key: ident) => {
        MultiKey::ConsumerKey(Consumer::$key)
    };
}

#[rustfmt::skip]
const KEYBOARD_LAYOUT: KeyboardLayout = KeyboardLayout {
    left: [
        // Layout 1
        [
            key!(NoEventIndicated), key!(Keyboard1), key!(Keyboard2), key!(Keyboard3), key!(Keyboard4), key!(Keyboard5),
            key!(NoEventIndicated), key!(Q), key!(W), key!(E), key!(R), key!(T),
            key!(NoEventIndicated), key!(A), key!(S), key!(D), key!(F), key!(G),
            key!(NoEventIndicated), key!(Z), key!(X), key!(C), key!(V), key!(B),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
        // Layout 2
        [
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
        // Layout 3
        [
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
    ],
    right: [
        // Layout 1
        [
            key!(Keyboard6), key!(Keyboard7), key!(Keyboard8), key!(Keyboard9), key!(Keyboard0), consumer!(PlayPause),
            key!(Y), key!(U), key!(I), key!(O), key!(P), key!(VolumeUp),
            key!(H), key!(J), key!(K), key!(L), key!(Semicolon), key!(Mute),
            key!(N), key!(M), key!(Comma), key!(Dot), key!(ForwardSlash), consumer!(VolumeDecrement),
            key!(NoEventIndicated), key!(Space), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
        // Layout 2
        [
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
        // Layout 3
        [
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
    ]
};

pub fn get_report(
    left_matrix: &[bool; 30],
    right_matrix: PrimitiveBitset<u32>,
    key_report: &mut FixedVec<Keyboard, 58>,
    media_report: &mut FixedVec<Consumer, 4>,
) {
    let rep_vec_prev_len = key_report.len;
    let media_prev_len = media_report.len;
    key_report.clear();
    media_report.clear();

    for i in 0..29 {
        if left_matrix[i] {
            match KEYBOARD_LAYOUT.left[0][i] {
                MultiKey::KeyboardKey(key) => key_report.push(key),
                MultiKey::ConsumerKey(key) => media_report.push(key),
            }
        }
    }

    for i in 0..29 {
        if right_matrix.get(i) {
            match KEYBOARD_LAYOUT.right[0][i] {
                MultiKey::KeyboardKey(key) => key_report.push(key),
                MultiKey::ConsumerKey(key) => media_report.push(key),
            }
        }
    }

    if rep_vec_prev_len > key_report.len {
        key_report.fill(Keyboard::NoEventIndicated, key_report.len);
    }

    if media_prev_len > media_report.len {
        media_report.fill(Consumer::Unassigned, media_report.len);
    }
}
