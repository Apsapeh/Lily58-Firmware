pub type KeybardMatrixLayout = [u8; 30];

pub struct KeyboardLayer {
    pub left: KeybardMatrixLayout,
    pub right: KeybardMatrixLayout,
}
