const IMM2D_IMPLEMENTATION: bool = true;
// Assuming the functions are defined in the imported module `immediate2d.rs`
// mod immediate2d will be used to import the functions
mod immediate2d {
    pub fn use_anti_aliasing() {
        // Dummy implementation for simulation
    }
    
    pub fn draw_circle(x: i32, y: i32, radius: i32, fill_color: &str, border_color: &str) {
        // Dummy implementation for simulation
    }
    
    pub fn wait(ms: i32) {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }
    
    pub fn clear() {
        // Dummy implementation for simulation
    }
}

use immediate2d::{use_anti_aliasing, draw_circle, wait, clear};

fn run() {
    let x = 80;
    let y = 60;
    let radius = 20;
    let delay = 1000;

    // Draw shapes with smooth edges
    use_anti_aliasing();

    // Loop forever
    loop {
        draw_circle(x, y, radius, "Yellow", "Brown");
        wait(delay);

        clear();
        wait(delay);
    }
}

fn main() {
    run();
}