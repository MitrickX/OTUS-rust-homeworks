use crate::cat::Cat;

mod cat;

fn print_cat(cat: Cat) {
    println!("Display {}", cat);
    println!("Debug {:?}", cat);
}

fn main() {
    let cat = Cat {
        age: 11,
        name: String::from("Fluffy"),
    };
    print_cat(cat);
    // println!("{}", cat) // нельзя использовать cat, потому что он был перенесен, что и требуется в задании
    // вся остальная демонстрация предоставлена в юнит тестах
}
