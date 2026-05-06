#include <iostream>

int main(){
    using namespace::std;
    string name;
    int age; 
    
    cout << "Enter your name: ";
    cin >> name;

    cout << endl << "Enter your age: ";
    cin >> age;

    cout << endl << "Welcome " << name << "You Are " << age << " Years Old!" << endl;
    
    return 0;
}