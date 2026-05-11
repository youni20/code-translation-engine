#include <iostream>
#include <vector>
#include <string>
#include <algorithm>

class Graph {
private:
    std::vector<std::vector<int>> adjacency;
    std::vector<std::string> labels;

public:
    Graph(int n) : adjacency(n), labels(n) {}

    void add_edge(int u, int v) {
        adjacency[u].push_back(v);
        adjacency[v].push_back(u);
    }

    void set_label(int node, const std::string& label) {
        labels[node] = label;
    }

    std::vector<int>& neighbors_mut(int node) {
        return adjacency[node];
    }

    const std::vector<int>& neighbors(int node) const {
        return adjacency[node];
    }

    void merge_neighbors(int u, int v) {
        auto& nu = neighbors_mut(u);
        const auto& nv = neighbors(v);
        for (int n : nv) {
            if (std::find(nu.begin(), nu.end(), n) == nu.end()) {
                nu.push_back(n);
            }
        }
    }
};

int main() {
    Graph g(5);
    g.add_edge(0, 1);
    g.add_edge(1, 2);
    g.add_edge(0, 2);
    g.set_label(0, "root");
    g.merge_neighbors(0, 1);
    for (int n : g.neighbors(0)) {
        std::cout << n << " ";
    }
    std::cout << std::endl;
    return 0;
}