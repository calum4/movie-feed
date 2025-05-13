pub const fn bool_true() -> bool {
    true
}

pub const fn bool_false() -> bool {
    false
}

pub fn vec_zero_size<T>() -> Vec<T> {
    Vec::with_capacity(0)
}
