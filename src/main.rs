
struct Human {
    pub age: i32,
    pub name: String
}

impl Human {
    fn cock(&self) {
        println!("{}", self.name)
    }
}

fn main() {
    let mut adam : Human = Human { age:11, name: String::from("Little billy")};

    adam.cock();
}
