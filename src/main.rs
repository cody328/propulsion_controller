use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct PropulsionController {
    fire_time: Option<Instant>,
}

impl PropulsionController {
    fn new() -> Self {
        PropulsionController { fire_time: None }
    }

    fn schedule_firing(&mut self, seconds: i32) {
        if seconds == -1 {
            // Cancel any pending firing
            self.fire_time = None;
            return;
        }

        // Schedule new firing
        let now = Instant::now();
        self.fire_time = Some(now + Duration::from_secs(seconds as u64));
    }

    fn time_until_firing(&self) -> Option<Duration> {
        self.fire_time.map(|time| {
            let now = Instant::now();
            if time > now {
                time.duration_since(now)
            } else {
                Duration::from_secs(0)
            }
        })
    }
}

fn main() {
    // Create a shared propulsion controller
    let controller = Arc::new(Mutex::new(PropulsionController::new()));
    
    // Clone for the firing thread
    let firing_controller = Arc::clone(&controller);
    
    // Spawn a thread to handle the firing
    let firing_thread = thread::spawn(move || {
        loop {
            // Check if we need to fire
            let duration_opt = {
                let controller = firing_controller.lock().unwrap();
                controller.time_until_firing()
            };
            
            match duration_opt {
                Some(duration) if duration.as_secs() == 0 => {
                    // Time to fire!
                    println!("firing now!");
                    
                    // Reset the firing time
                    let mut controller = firing_controller.lock().unwrap();
                    controller.fire_time = None;
                }
                Some(duration) => {
                    // Wait a bit before checking again
                    let sleep_time = std::cmp::min(duration, Duration::from_millis(100));
                    thread::sleep(sleep_time);
                }
                None => {
                    // No firing scheduled, just wait a bit
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    });
    
    // Process input commands
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(input) = line {
            if let Ok(seconds) = input.trim().parse::<i32>() {
                let mut controller = controller.lock().unwrap();
                controller.schedule_firing(seconds);
            }
        }
    }
    
    // Wait for the firing thread (this won't actually happen in normal execution)
    let _ = firing_thread.join();
}
