use std::vec::Vec;
use std::string::String;

struct Graph {
    adjacency: Vec<Vec<i32>>,
    labels: Vec<String>,
}

impl Graph {
    fn new(n: usize) -> Self {
        Self {
            adjacency: vec![Vec::new(); n],
            labels: vec![String::new(); n],
        }
    }

    fn add_edge(&mut self, u: usize, v: usize) {
        self.adjacency[u].push(v as i32);
        self.adjacency[v].push(u as i32);
    }

    fn set_label(&mut self, node: usize, label: String) {
        self.labels[node] = label;
    }

    fn neighbors_mut(&mut self, node: usize) -> &mut Vec<i32> {
        &mut self.adjacency[node]
    }

    fn neighbors(&self, node: usize) -> &Vec<i32> {
        &self.adjacency[node]
    }

    fn merge_neighbors(&mut self, u: usize, v: usize) {
        let nu = self.neighbors_mut(u);
        let nv = self.neighbors(v).clone(); // Clone the immutable reference to avoid borrowing issues
        for &n in nv.iter() {
            if !nu.contains(&n) {
                nu.push(n);
            }
        }
    }
}

fn main() {
    let mut g = Graph::new(5);
    g.add_edge(0, 1);
    g.add_edge(1, 2);
    g.add_edge(0, 2);
    g.set_label(0, String::from("root"));
    g.merge_neighbors(0, 1);
    for &n in g.neighbors(0) {
        print!("{} ", n);
    }
    println!();
}