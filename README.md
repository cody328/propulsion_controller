# Satellite Propulsion Controller

This program manages a satellite's propulsion system by accepting commands with relative times to fire the propulsion.

---

## Problem Overview

The flight computer should accept a command with a relative time specifying when to fire the propulsion. Once that time has elapsed, the program prints:

```
firing now!
```

If another command arrives before firing, the most recently received command overwrites the previous one.

If a relative time of `-1` is given, any outstanding commands to fire are cancelled.

The flight computer can fire the thruster multiple times during a single program execution.

---

## Approach

Implemented in Rust with this design:

- Accept commands specifying relative times (in seconds) to fire propulsion.
- When time elapses, print `"firing now!"`.
- New commands before firing overwrite previous commands.
- A command with time `-1` cancels pending commands.
- Support multiple firings in one run.
- Communication via standard input/output or TCP (port 8124).
- Uses a dedicated thread for timing without blocking input handling.
- Propulsion controller state is shared using a mutex for thread safety.

---

## How to Build and Run

### Prerequisites

- Rust 1.56.0 or later
- Cargo (comes with Rust)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Build

```bash
cargo build --release
```

### Running



Run the TCP server:

```bash
cargo run --release --bin propulsion_controller_tcp
```

This starts a TCP server on port 8124.

Use the provided client script to connect:

```bash
python3 propulsion_tcp_client.py
```

Type commands (relative times in seconds) into the client terminal.

---

## Example Usage

### Standard Input Mode

```
cargo run --release --bin propulsion_controller_tcp

New client connected: 127.0.0.1:48534
Received command: 15 seconds
Received command: 30 seconds
firing now!
Received command: -1 (Cancelling pending firing)
Received command: 10 seconds
firing now!
```

Explanation:

- Command to fire in 15 seconds received.
- Then command to fire in 30 seconds received (overrides previous).
- After 30 seconds: prints `"firing now!"`
- Command for 5 seconds received.
- After 5 seconds: prints `"firing now!"`
- Command `-1` cancels any pending commands.
- Command for 10 seconds received.
- After 10 seconds: prints `"firing now!"`


## Implementation Details

- **PropulsionController:** Manages scheduling and firing propulsion.
- **Main Thread:** Handles input (stdin or TCP).

State is protected with a mutex to safely share between the input and firing threads.

For the TCP server, each client connection runs in a separate thread. When propulsion fires, all connected clients receive `"firing now!"`.
