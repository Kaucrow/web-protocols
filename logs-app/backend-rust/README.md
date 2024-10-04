# WebSocket Logging - Rust Backend

A WebSocket server which parses a "log frame" formatted as `init^cmd^data^endData^close`, and logs its contents to a file.

Run `cargo rustdoc --help` to view the crate documentation.

## Usage
Connect via WebSocket (e.g., with Postman) to `ws://localhost:8080/ws` to test.