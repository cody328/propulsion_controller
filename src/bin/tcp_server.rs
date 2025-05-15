use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct PropulsionController {
    fire_time: Option<Instant>,
    clients: Vec<TcpStream>,
}

impl PropulsionController {
    fn new() -> Self {
        PropulsionController { 
            fire_time: None,
            clients: Vec::new(),
        }
    }

    fn schedule_firing(&mut self, seconds: i32) {
        if seconds == -1 {
            // Cancel any pending firing
            println!("Received command: {} (Cancelling pending firing)", seconds);
            self.fire_time = None;
            return;
        }

        // Schedule new firing
        println!("Received command: {} seconds", seconds);
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

    fn add_client(&mut self, client: TcpStream) {
        self.clients.push(client);
    }

    fn broadcast_message(&mut self, message: &str) {
        let mut failed_clients = Vec::new();
        
        for (index, client) in self.clients.iter_mut().enumerate() {
            if let Err(_) = client.write_all(message.as_bytes()) {
                failed_clients.push(index);
            }
        }
        
        // Remove failed clients in reverse order to avoid index issues
        for index in failed_clients.iter().rev() {
            self.clients.remove(*index);
        }
    }
}

fn handle_client(stream: TcpStream, controller: Arc<Mutex<PropulsionController>>) {
    println!("New client connected: {:?}", stream.peer_addr().unwrap_or_else(|_| "Unknown".parse().unwrap()));
    
    // Clone the stream for the controller
    let client_stream = match stream.try_clone() {
        Ok(client) => client,
        Err(_) => return,
    };
    
    // Add the client to the controller
    {
        let mut controller = controller.lock().unwrap();
        controller.add_client(client_stream);
    }
    
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    
    while let Ok(bytes_read) = reader.read_line(&mut line) {
        if bytes_read == 0 {
            // Connection closed
            println!("Client disconnected");
            break;
        }
        
        if let Ok(seconds) = line.trim().parse::<i32>() {
            let mut controller = controller.lock().unwrap();
            controller.schedule_firing(seconds);
        }
        
        line.clear();
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
                    let message = "firing now!\n";
                    println!("{}", message.trim());
                    
                    // Broadcast to all clients
                    let mut controller = firing_controller.lock().unwrap();
                    controller.broadcast_message(message);
                    
                    // Reset the firing time
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
    
    // Set up TCP server
    let listener = TcpListener::bind("127.0.0.1:8124").expect("Failed to bind to port 8124");
    println!("TCP server listening on port 8124");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let client_controller = Arc::clone(&controller);
                thread::spawn(move || {
                    handle_client(stream, client_controller);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
    
    // Wait for the firing thread
    let _ = firing_thread.join();
}
