# TCP/UDP/WebSockets Logger App

A logger app that logs formatted messages referred to as "log frames" using the following frame structure:

```
init^cmd^data^endData^close
```

Where:
- `cmd` represents the log type (e.g., `info`, `debug`, `warn`, `error`).
- `data` contains the actual log message.

The project has two backend servers that can be used:
1. **Node.js server** (TCP and UDP server versions)
2. **Rust server using Actix-web** (WebSockets and UDP server versions)

## Features

- Receives log frames via TCP and UDP on the node server, and over WebSockets and UDP on the rust server.
- Logs the following types of messages:
  - `info`
  - `debug`
  - `warn`
  - `error`

  (Any other type of message gets logged with a default type)