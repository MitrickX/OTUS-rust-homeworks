#[macro_export]
macro_rules! my_macro {
    ($($i:ident),+ $(,)?) => {
        ($(
            $i(),
        )+)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_macro_works() {
        fn foo() -> i32 {
            1 + 2
        }

        fn bar() -> &'static str {
            "Hello"
        }

        fn bas() -> [i32; 3] {
            [1, 2, 3]
        }

        let (foo_result, bar_result, bas_result) = my_macro!(foo, bar, bas);
        assert_eq!(foo_result, foo());
        assert_eq!(bar_result, bar());
        assert_eq!(bas_result, bas());
    }
}
