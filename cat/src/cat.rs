use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign};

#[derive(Debug)]
enum Pet {
    #[allow(dead_code)]
    Dog(u8, String),
    Cat(u8, String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cat {
    pub age: u8,
    pub name: String,
}

impl Display for Cat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cat(name: {}, age: {})", self.name, self.age)
    }
}

impl TryFrom<Pet> for Cat {
    type Error = String;
    fn try_from(pet: Pet) -> Result<Self, Self::Error> {
        match pet {
            Pet::Cat(age, name) => Ok(Cat { age, name }),
            p => Err(format!("can't convert {:?} to Cat struct", p)),
        }
    }
}

impl From<Cat> for Pet {
    fn from(cat: Cat) -> Self {
        Pet::Cat(cat.age, cat.name)
    }
}

impl AsRef<str> for Cat {
    fn as_ref(&self) -> &str {
        self.name.as_str()
    }
}

impl Add<u8> for Cat {
    type Output = Cat;

    fn add(self, rhs: u8) -> Self::Output {
        Cat {
            name: self.name,
            age: self.age + rhs,
        }
    }
}

impl AddAssign<u8> for Cat {
    fn add_assign(&mut self, rhs: u8) {
        self.age += rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_works() {
        let cat = Cat {
            age: 13,
            name: String::from("Fluffy"),
        };
        let clone_cat = cat.clone();

        assert_eq!(cat, clone_cat);
    }

    #[test]
    fn format_works() {
        let cat = Cat {
            age: 13,
            name: String::from("Fluffy"),
        };
        let formatted = format!("debug = {:?}, display = {}", cat, cat);

        assert_eq!(
            String::from(
                "debug = Cat { age: 13, name: \"Fluffy\" }, display = Cat(name: Fluffy, age: 13)"
            ),
            formatted
        )
    }

    #[test]
    fn from_pet_works() {
        let pet1 = Pet::Cat(8, String::from("Daisy"));
        let pet2 = Pet::Dog(3, String::from("Toby"));
        let cat1: Result<Cat, String> = pet1.try_into();
        let cat2: Result<Cat, String> = pet2.try_into();

        assert_eq!(true, cat1.is_ok());
        assert_eq!(
            Cat {
                age: 8,
                name: String::from("Daisy")
            },
            cat1.unwrap()
        );

        assert_eq!(true, cat2.is_err());
        assert_eq!(
            String::from("can't convert Dog(3, \"Toby\") to Cat struct"),
            cat2.unwrap_err()
        );
    }

    #[test]
    fn add_works() {
        let cat = Cat {
            age: 7,
            name: String::from("Bella"),
        };
        let new_cat = cat + 3;

        assert_eq!(
            Cat {
                age: 10,
                name: String::from("Bella")
            },
            new_cat
        );

        let mut cat = Cat {
            age: 2,
            name: String::from("Misty"),
        };
        cat += 1;

        assert_eq!(
            Cat {
                age: 3,
                name: String::from("Misty")
            },
            cat
        );
    }
}
