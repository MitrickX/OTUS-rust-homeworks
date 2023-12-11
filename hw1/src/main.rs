fn double_int32(x: i32) -> i32 {
    x * 2
}

fn double_int64(x: i32) -> i64 {
    x as i64 * 2
}

fn double_float32(x: f32) -> f32 {
    x * 2.
}

fn double_float64(x: f32) -> f64 {
    x as f64 * 2.
}

fn int_plus_float_to_float(x: i32, y: f32) -> f64 {
    x as f64 * y as f64
}

fn int_plus_float_to_int(x: i32, y: f32) -> i64 {
    (x as f64 + y as f64) as i64
}

fn tuple_sum(p: (i32, i32)) -> i32 {
    p.0 + p.1
}

fn array_sum(a: [i32; 3]) -> i32 {
    a.iter().sum()
}

fn main() {
    println!("HW1:");
    println!("double_int32({}) = {}", 10, double_int32(10));
    println!("double_int64({}) = {}", i32::MAX, double_int64(i32::MAX));
    println!("double_float32({}) = {}", 10.1, double_float32(10.1));
    println!("double_float64({}) = {}", f32::MAX, double_float64(f32::MAX));
    println!("int_plus_float_to_float({}, {}) = {}", 3, f32::MAX, int_plus_float_to_float(3, f32::MAX));
    println!("int_plus_float_to_int({}, {}) = {}", 3, f32::MAX, int_plus_float_to_int(3, f32::MAX));
    println!("tuple_sum(({}, {})) = {}", 3, 5, tuple_sum((3, 5)));
    println!("array_sum([{}, {}, {}]) = {}", 1, 2, 3, array_sum([1, 2, 3]));
}
