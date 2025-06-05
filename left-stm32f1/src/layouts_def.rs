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
    left: [KeybardMatrixLayout; 2],
    right: [KeybardMatrixLayout; 4],
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
            key!(Escape), key!(Keyboard1), key!(Keyboard2), key!(Keyboard3), key!(Keyboard4), key!(Keyboard5),
            key!(Tab), key!(Q), key!(W), key!(E), key!(R), key!(T),
            key!(CapsLock), key!(A), key!(S), key!(D), key!(F), key!(G),
            key!(LeftShift), key!(Z), key!(X), key!(C), key!(V), key!(B),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(LeftAlt), key!(LeftControl), key!(Space), key!(LeftGUI),
        ],
        // Layout 2
        [
            key!(Mute), key!(VolumeDown), key!(VolumeUp), consumer!(PlayPause), consumer!(Rewind), consumer!(FastForward),
            key!(F1), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(Copy), key!(Paste), key!(Cut),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(LeftAlt), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
    ],
    right: [
        // Layout 1
        [
            key!(Keyboard6), key!(Keyboard7), key!(Keyboard8), key!(Keyboard9), key!(Keyboard0), key!(Minus),
            key!(Y), key!(U), key!(I), key!(O), key!(P), key!(DeleteBackspace),
            key!(H), key!(J), key!(K), key!(L), key!(Semicolon), key!(ReturnEnter),
            key!(N), key!(M), key!(Comma), key!(Dot), key!(ForwardSlash), key!(RightShift),
            key!(Grave), key!(Space), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
        // Layout 2
        [
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(Equal), key!(Backslash), key!(LeftBrace), key!(RightBrace), key!(Apostrophe), key!(DeleteForward),
            key!(NoEventIndicated), key!(LeftArrow), key!(DownArrow), key!(UpArrow), key!(RightArrow), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(Home), key!(End), key!(PageUp), key!(PageDown), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
        // Layout 3
        [
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(Keypad7), key!(Keypad8), key!(Keypad9), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(Keypad4), key!(Keypad5), key!(Keypad6), key!(KeypadEnter), key!(NoEventIndicated),
            key!(Keypad0), key!(Keypad1), key!(Keypad2), key!(Keypad3), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
        // Layout 4
        [
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(Keypad7), key!(Keypad8), key!(Keypad9), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(Keypad4), key!(Keypad5), key!(Keypad6), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(Keypad1), key!(Keypad2), key!(Keypad3), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
    ]
};

/*
    +   # ъ
    \   \ э
    [   / ю
    ]   @ ж
    '   - '
    
    prscr
    numlock
    
*/

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
    
    let left_layer = get_left_layer(&left_matrix);
    let right_layer = if left_layer == 1 {
        // For ergonomic meta + alt + <arrows>
        1
    } else {
        get_right_layer(&right_matrix)
    };

    for i in 0..30 {
        if left_matrix[i] {
            match KEYBOARD_LAYOUT.left[left_layer][i] {
                MultiKey::KeyboardKey(key) => key_report.push(key),
                MultiKey::ConsumerKey(key) => media_report.push(key),
            }
        }
    }
    

    for i in 0..29 {
        if right_matrix.get(i) {
            match KEYBOARD_LAYOUT.right[right_layer][i] {
                MultiKey::KeyboardKey(key) => key_report.push(key),
                MultiKey::ConsumerKey(key) => media_report.push(key),
            }
        }
    }
    
    // Meta + Alt
    if left_layer == 1 && left_matrix[26] {
        key_report.push(Keyboard::LeftGUI);
    }

    if rep_vec_prev_len > key_report.len {
        key_report.fill(Keyboard::NoEventIndicated, key_report.len);
    }

    if media_prev_len > media_report.len {
        media_report.fill(Consumer::Unassigned, media_report.len);
    }
}


fn get_left_layer(matrix: &[bool; 30]) -> usize {
    if matrix[25] {
        1
    } else {
        0
    }
}

fn get_right_layer(matrix: &PrimitiveBitset<u32>) -> usize {
    if matrix.get(26) {
        1
    } else if matrix.get(27) {
        2
    } else if matrix.get(28) {
        3
    } else {
        0
    }
}