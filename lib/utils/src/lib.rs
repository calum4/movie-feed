#[macro_export]
macro_rules! const_assert {
    ($($tt:tt)*) => {
        const _: () = assert!($($tt)*);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_const_assert() {
        const_assert!(true);
    }
}
