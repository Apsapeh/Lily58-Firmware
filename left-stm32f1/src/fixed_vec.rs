pub struct FixedVec<T, const N: usize> {
    pub data: [T; N],
    pub len: usize,
}

impl<T: Copy, const N: usize> FixedVec<T, N> {
    pub fn new(default: T) -> FixedVec<T, N> {
        FixedVec {
            data: [default; N],
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn fill(&mut self, value: T, start_index: usize) {
        for i in start_index..N {
            self.data[i] = value;
        }
    }

    pub fn push(&mut self, value: T) {
        if self.len >= N {
            return;
        }

        self.data[self.len] = value;
        self.len += 1;
    }
}
