// #![no_main]
// #![no_std]

// use core::ops::{BitAnd, BitOr, Not, Shl, Shr};

// pub struct PrimitiveBitset<T> {
//     data: T,
// }

// trait AllowedTypes {}
// impl AllowedTypes for u8 {}
// impl AllowedTypes for u16 {}
// impl AllowedTypes for u32 {}
// impl AllowedTypes for u64 {}

// /*pub enum BitValue <T: num::Integer> {
//     One(T) = T::one()
// }*/

// impl<
//         T: num::Integer
//             + Copy
//             + BitAnd<Output = T>
//             + BitOr<Output = T>
//             + Shl<Output = T>
//             + Shr<Output = T>
//             + Not<Output = T>,
//     > PrimitiveBitset<T>
// {
//     pub fn new(data: T) -> Self {
//         Self { data }
//     }

//     #[inline]
//     pub fn set(&mut self, idx: T, value: T) {
//         //let value = if value { T::one() } else { T::zero() };
//         self.data = (self.data & !(T::one() << idx)) | ((value & T::one()) << idx)
//     }

//     #[inline]
//     pub fn get(&self, idx: T) -> bool {
//         let val = (self.data >> idx) & T::one();
//         val == T::one()
//     }

//     #[inline]
//     pub fn set_raw(&mut self, data: T) {
//         self.data = data;
//     }

//     #[inline]
//     pub fn get_raw(&self) -> T {
//         self.data
//     }

//     #[inline]
//     pub fn clear(&mut self) {
//         self.data.set_zero();
//     }
// }
// 
// 
#![no_std]

use core::ops::{BitAnd, BitOr, Not, Shl, Shr};

pub trait BitsetWord:
    Copy
    + Default
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + Not<Output = Self>
    + PartialEq
{
    fn one() -> Self;
    fn zero() -> Self;
}

impl BitsetWord for u8 {
    #[inline(always)] fn one() -> Self { 1 }
    #[inline(always)] fn zero() -> Self { 0 }
}
impl BitsetWord for u16 {
    #[inline(always)] fn one() -> Self { 1 }
    #[inline(always)] fn zero() -> Self { 0 }
}
impl BitsetWord for u32 {
    #[inline(always)] fn one() -> Self { 1 }
    #[inline(always)] fn zero() -> Self { 0 }
}
impl BitsetWord for u64 {
    #[inline(always)] fn one() -> Self { 1 }
    #[inline(always)] fn zero() -> Self { 0 }
}

pub struct PrimitiveBitset<T: BitsetWord> {
    data: T,
}

impl<T: BitsetWord> PrimitiveBitset<T> {
    #[inline(always)]
    pub fn new(data: T) -> Self {
        Self { data }
    }

    #[inline(always)]
    pub fn set(&mut self, idx: usize, value: bool) {
        let mask = T::one() << idx;
        let bit = if value { mask } else { T::zero() };
        self.data = (self.data & !mask) | bit;
    }

    #[inline(always)]
    pub fn get(&self, idx: usize) -> bool {
        ((self.data >> idx) & T::one()) == T::one()
    }

    #[inline(always)]
    pub fn set_raw(&mut self, data: T) {
        self.data = data;
    }

    #[inline(always)]
    pub fn get_raw(&self) -> T {
        self.data
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.data = T::zero();
    }
}

