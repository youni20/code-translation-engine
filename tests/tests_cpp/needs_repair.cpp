#include <iostream>
#include <vector>
#include <string>
#include <memory>
#include <map>

class Inventory {
private:
    std::map<std::string, std::shared_ptr<std::vector<int>>> items;

public:
    void add(const std::string& name, int quantity) {
        if (items.find(name) == items.end()) {
            items[name] = std::make_shared<std::vector<int>>();
        }
        items[name]->push_back(quantity);
    }

    int total(const std::string& name) const {
        auto it = items.find(name);
        if (it == items.end()) return 0;
        int sum = 0;
        for (int q : *(it->second)) sum += q;
        return sum;
    }
};

int main() {
    Inventory inv;
    inv.add("apples", 10);
    inv.add("apples", 5);
    inv.add("oranges", 3);
    std::cout << "Apples: " << inv.total("apples") << std::endl;
    std::cout << "Oranges: " << inv.total("oranges") << std::endl;
    return 0;
}