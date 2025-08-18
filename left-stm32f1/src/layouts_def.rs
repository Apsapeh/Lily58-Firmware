use crate::fixed_vec::FixedVec;
use shared_src::PrimitiveBitset;
use static_assertions::const_assert_eq;
use usbd_human_interface_device::{
    page::{Consumer, Keyboard},
};

#[derive(Copy, Clone, PartialEq)]
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

const LEFT_FN: usize = 25;
//const LEFT_SHIFT: usize = 18;

const RIGHT_FN_1: usize = 26;
const RIGHT_FN_2: usize = 27;
const RIGHT_FN_3: usize = 28;


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
            key!(Mute), key!(VolumeDown), key!(VolumeUp), consumer!(PlayPause), consumer!(ScanPreviousTrack), consumer!(ScanNextTrack),
            key!(PrintScreen), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(Copy), key!(Paste), key!(Cut),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(LeftAlt), key!(KeypadNumLockAndClear), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
    ],
    right: [
        // Layout 1
        [
            key!(Keyboard6), key!(Keyboard7), key!(Keyboard8), key!(Keyboard9), key!(Keyboard0), key!(Minus),
            key!(Y), key!(U), key!(I), key!(O), key!(P), key!(DeleteBackspace),
            key!(H), key!(J), key!(K), key!(L), key!(Semicolon), key!(ReturnEnter),
            key!(N), key!(M), key!(Comma), key!(Dot), key!(ForwardSlash), key!(RightShift),
            key!(RightAlt), key!(Space), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
        // Layout 2
        [
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(Equal), key!(Backslash), key!(LeftBrace), key!(RightBrace), key!(Apostrophe), key!(DeleteForward),
            key!(Grave), key!(LeftArrow), key!(DownArrow), key!(UpArrow), key!(RightArrow), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(Home), key!(End), key!(PageUp), key!(PageDown), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
        // Layout 3
        [
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(Keypad7), key!(Keypad8), key!(Keypad9), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(Keypad4), key!(Keypad5), key!(Keypad6), key!(KeypadEnter), key!(KeypadEnter),
            key!(Keypad0), key!(Keypad1), key!(Keypad2), key!(Keypad3), key!(KeypadDot), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
        // Layout 4
        [
            key!(F1), key!(F2), key!(F3), key!(F4), key!(F5), key!(F6),
            key!(F7), key!(F8), key!(F9), key!(F10), key!(F11), key!(F12),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
            consumer!(ALCalculator), consumer!(ALFileBrowser), consumer!(ALInternetBrowser), consumer!(ALCommandLineProcessorRun), key!(NoEventIndicated), key!(NoEventIndicated),
            key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated), key!(NoEventIndicated),
        ],
    ]
};

static mut prev_left_layer: usize = 0;
static mut prev_right_layer: usize = 0;
static mut blocked_right_matrix: [bool; 30] = [false; 30];
static mut blocked_left_matrix: [bool; 30] = [false; 30];

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

    unsafe {
        if left_layer != prev_left_layer {
            prev_left_layer = left_layer;
            for i in 0..30 {
                blocked_left_matrix[i] = left_matrix[i];
            }
        }

        if right_layer != prev_right_layer {
            prev_right_layer = right_layer;
            for i in 0..29 {
                blocked_right_matrix[i] = right_matrix.get(i);
            }
        }

        for i in 0..30 {
            if !blocked_left_matrix[i] && left_matrix[i] {
                match KEYBOARD_LAYOUT.left[left_layer][i] {
                    MultiKey::KeyboardKey(key) => key_report.push(key),
                    MultiKey::ConsumerKey(key) => media_report.push(key),
                }
            } else if !left_matrix[i] {
                blocked_left_matrix[i] = false;
            }
        }

        for i in 0..29 {
            if !blocked_right_matrix[i] && right_matrix.get(i) {
                match KEYBOARD_LAYOUT.right[right_layer][i] {
                    MultiKey::KeyboardKey(key) => key_report.push(key),
                    MultiKey::ConsumerKey(key) => media_report.push(key),
                }
            } else if !right_matrix.get(i) {
                blocked_right_matrix[i] = false;
            }
        }
    }

    // Meta + Alt + <arrows>
    if left_layer == 1 && (right_matrix.get(13) | right_matrix.get(16)) {
        key_report.push(Keyboard::LeftGUI);
        key_report.push(Keyboard::LeftAlt); // If first was pressed an ALT and only after a GUI, ALT would be blocked
    }
    
    if left_layer == 1 && [19, 20, 21, 22].iter().any(|&i| right_matrix.get(i)) {
        key_report.push(Keyboard::LeftGUI);
        key_report.push(Keyboard::LeftShift);
    }

    if rep_vec_prev_len > key_report.len {
        key_report.fill(Keyboard::NoEventIndicated, key_report.len);
    }

    if media_prev_len > media_report.len {
        media_report.fill(Consumer::Unassigned, media_report.len);
    }
}

fn get_left_layer(matrix: &[bool; 30]) -> usize {
    if matrix[LEFT_FN] {
        1
    } else {
        0
    }
}

fn get_right_layer(matrix: &PrimitiveBitset<u32>) -> usize {
    if matrix.get(RIGHT_FN_1) {
        1
    } else if matrix.get(RIGHT_FN_2) {
        2
    } else if matrix.get(RIGHT_FN_3) {
        3
    } else {
        0
    }
}