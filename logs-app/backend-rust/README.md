# Logger App - Rust Backend

A logger app with both a WebSocket and UDP server which parses a "log frame" formatted as `init^cmd^data^endData^close`, and logs its contents to a file.

Run `cargo rustdoc --open` to view the crate documentation.

## Usage
* WebSocket: `ws://localhost:8080/ws`. Recommended testing tool: Postman.
* UDP: `localhost:8081`. Recommended testing tool: Netcat.