#include <iostream>

int two_sum(int x, int y){
    int result = x + y;
    return result;
}


int main(){
    int x;
    int y;
    std::cout << "What two numbers would you like to add?\n" << "Number 1: ";
    std::cin >> x;
    std::cout << "Number 2: ";
    std::cin >> y;
    std::cout << std::endl;
    
    int result = two_sum(x, y);
    std::cout << "The Result: " << result << std::endl;
    return 0;
}