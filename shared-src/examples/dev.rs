use shared_src::PrimitiveBitset;
use std::time::Instant;

const PATTERN: [bool; 29] = [
    true, false, true, true, false, false, true, false, true, true, false, true, false, false,
    true, true, true, false, false, true, false, false, true, true, false, true, false, true,
    false,
];

fn main() {
    let mut bitset = PrimitiveBitset::new(0u64);

    println!("x: {:0>32b}", bitset.get_raw());
    println!("bit: {}", bitset.get(9));
    bitset.set(9, true);
    println!("x: {:0>32b}", bitset.get_raw());
    println!("bit: {}", bitset.get(9));

    let start = Instant::now();
    for _ in 0..10_000_000 {
        for i in 0..29 {
            unsafe {
                bitset.set(i, *PATTERN.get_unchecked(i as usize));
                std::hint::black_box(&bitset);
            }
        }
    }
    let duration = start.elapsed();
    println!("Время выполнения: {:.2?}", duration);

    let mut bool_arr = [false; 32];
    let mut out: u32 = 0;
    let start = Instant::now();
    for _ in 0..10_000_000 {
        for i in 0..29 {
            unsafe {
                //bitset.set(i, *PATTERN.get_unchecked(i as usize));
                bool_arr[i as usize] = *PATTERN.get_unchecked(i as usize);
                std::hint::black_box(&bool_arr);
            }
        }

        for i in 0..29 {
            unsafe {
                out = (out & !(1 << i as usize))
                    | ((*bool_arr.get_unchecked(i as usize) as u32) << (i as usize));
            }
            std::hint::black_box(&out);
        }
    }
    let duration = start.elapsed();
    println!("Время выполнения: {:.2?}", duration);

    let mut out: u32 = 0;
    let start = Instant::now();
    for _ in 0..10_000_000 {
        for i in 0..29 {
            unsafe {
                out = (out & !(1 << i as usize))
                    | ((*PATTERN.get_unchecked(i as usize) as u32) << (i as usize));
            }
            std::hint::black_box(&out);
        }
    }
    let duration = start.elapsed();
    println!("Время выполнения: {:.2?}", duration);
}
