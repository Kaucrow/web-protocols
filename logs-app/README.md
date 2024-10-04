# WebSocket Logging App

A WebSocket app that logs formatted messages referred here as "log frames" using the following frame structure:

```
init^cmd^data^endData^close
```

Where:
- `cmd` represents the log type (e.g., `info`, `debug`, `warn`, `error`).
- `data` contains the actual log message.

The project has two backend servers that can be used:
1. **Node.js server**
2. **Rust server using Actix-web**

## Features

- Receives log frames via WebSocket.
- Logs the following types of messages:
  - `info`
  - `debug`
  - `warn`
  - `error`

  (Any other type of message gets logged with a default type)
