#[macro_export]
macro_rules! new_default {
    ($kind:ty, $name:ident) => {
        pub const fn $name<const V: $kind>() -> $kind {
            V
        }
    };
}

new_default!(u8, default_u8);
new_default!(u16, default_u16);
new_default!(u32, default_u32);
new_default!(u64, default_u64);
new_default!(u128, default_u128);
new_default!(usize, default_usize);

new_default!(i8, default_i8);
new_default!(i16, default_i16);
new_default!(i32, default_i32);
new_default!(i64, default_i64);
new_default!(i128, default_i128);
new_default!(isize, default_isize);

new_default!(bool, default_bool);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default_unsized_ints() {
        assert_eq!(default_u8::<{ u8::MAX }>(), u8::MAX);
        assert_eq!(default_u16::<{ u16::MAX }>(), u16::MAX);
        assert_eq!(default_u32::<{ u32::MAX }>(), u32::MAX);
        assert_eq!(default_u64::<{ u64::MAX }>(), u64::MAX);
        assert_eq!(default_u128::<{ u128::MAX }>(), u128::MAX);
        assert_eq!(default_usize::<{ usize::MAX }>(), usize::MAX);
    }

    #[test]
    fn test_default_sized_ints() {
        assert_eq!(default_i8::<{ i8::MAX }>(), i8::MAX);
        assert_eq!(default_i16::<{ i16::MAX }>(), i16::MAX);
        assert_eq!(default_i32::<{ i32::MAX }>(), i32::MAX);
        assert_eq!(default_i64::<{ i64::MAX }>(), i64::MAX);
        assert_eq!(default_i128::<{ i128::MAX }>(), i128::MAX);
        assert_eq!(default_isize::<{ isize::MAX }>(), isize::MAX);
    }

    #[test]
    fn test_default_bool() {
        assert!(default_bool::<true>());
        assert!(!default_bool::<false>());
    }
}
