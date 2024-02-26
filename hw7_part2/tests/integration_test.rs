use hw7_part2::my_macro;

#[test]
fn my_macro_works() {
    fn fo() -> i32 {
        1 + 2
    }

    fn fooo() -> [i32; 3] {
        [1, 2, 3]
    }

    let (fo_result, fooo_result) = my_macro!("fo", "foo", "fooo");
    assert_eq!(fo_result, fo());
    assert_eq!(fooo_result, fooo());
}
