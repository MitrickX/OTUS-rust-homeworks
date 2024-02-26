pub fn foo() -> i32 {
    1 + 2
}

pub fn bar() -> &'static str {
    "Hello"
}

pub fn bas() -> [i32; 3] {
    [1, 2, 3]
}

fn main() {
    let (foo_result, bar_result, bas_result) = hw7::my_macro!(foo, bar, bas);
    println!("foo_result = {:?}", foo_result);
    println!("bar_result = {:?}", bar_result);
    println!("bas_result = {:?}", bas_result);

    fn fo() -> i32 {
        123
    }

    fn fooo() -> &'static str {
        "Rust"
    }

    let (fo_res, fooo_res) = hw7_part2::my_macro!("fo", "foo", "fooo");
    println!("fo_res = {}", fo_res);
    println!("fooo_res = {}", fooo_res);
}
