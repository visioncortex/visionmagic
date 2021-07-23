pub type Bitmask = Vec<u32>;
#[inline]
pub fn new(size: usize) -> Bitmask {
    vec![
        0;
        if size % 32 == 0 {
            size / 32
        } else {
            size / 32 + 1
        }
    ]
}

#[inline]
pub fn get(v: &[u32], i: usize) -> bool {
    let w = i / 32;
    let b = i % 32;
    (v[w] & (1 << b)) != 0
}

#[inline]
pub fn set(v: &mut [u32], i: usize) {
    let w = i / 32;
    let b = i % 32;
    v[w] |= 1 << b;
}

#[inline]
pub fn unset(v: &mut [u32], i: usize) {
    let w = i / 32;
    let b = i % 32;
    v[w] &= !(1 << b);
}
