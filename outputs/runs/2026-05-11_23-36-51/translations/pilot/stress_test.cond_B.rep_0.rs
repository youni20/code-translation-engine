use std::collections::HashSet;

struct Graph {
    adjacency: Vec<Vec<i32>>,
    labels: Vec<String>,
}

impl Graph {
    fn new(n: usize) -> Self {
        Graph {
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
        let nv = self.neighbors(v);
        let existing: HashSet<_> = nu.iter().copied().collect();
        
        let new_neighbors: Vec<i32> = nv.iter().filter(|&&n| !existing.contains(&n)).copied().collect();
        nu.extend(new_neighbors);
    }
}

fn main() {
    let mut g = Graph::new(5);
    g.add_edge(0, 1);
    g.add_edge(1, 2);
    g.add_edge(0, 2);
    g.set_label(0, "root".to_string());
    g.merge_neighbors(0, 1);
    for &n in g.neighbors(0) {
        print!("{} ", n);
    }
    println!();
}