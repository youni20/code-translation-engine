use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct Inventory {
    items: HashMap<String, Arc<Mutex<Vec<i32>>>>,
}

impl Inventory {
    fn new() -> Self {
        Inventory {
            items: HashMap::new(),
        }
    }

    fn add(&mut self, name: &str, quantity: i32) {
        self.items
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(Vec::new())))
            .lock()
            .unwrap()
            .push(quantity);
    }

    fn total(&self, name: &str) -> i32 {
        match self.items.get(name) {
            Some(vec) => vec.lock().unwrap().iter().sum(),
            None => 0,
        }
    }
}

fn main() {
    let mut inv = Inventory::new();
    inv.add("apples", 10);
    inv.add("apples", 5);
    inv.add("oranges", 3);
    println!("Apples: {}", inv.total("apples"));
    println!("Oranges: {}", inv.total("oranges"));
}