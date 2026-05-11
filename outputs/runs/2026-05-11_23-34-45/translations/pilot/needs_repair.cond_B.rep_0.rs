use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;

struct Inventory {
    items: HashMap<String, Arc<Vec<i32>>>,
}

impl Inventory {
    fn new() -> Self {
        Inventory {
            items: HashMap::new(),
        }
    }

    fn add(&mut self, name: &str, quantity: i32) {
        if !self.items.contains_key(name) {
            self.items.insert(name.to_string(), Arc::new(Vec::new()));
        }
        if let Some(vec) = self.items.get_mut(name) {
            Arc::get_mut(vec).unwrap().push(quantity);
        }
    }

    fn total(&self, name: &str) -> i32 {
        match self.items.get(name) {
            Some(vec) => vec.iter().sum(),
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