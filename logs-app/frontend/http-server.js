import express from "express";
import bodyParser from "body-parser";
import path from "path";
import { fileURLToPath } from "url";
import { sendTcpMessage, sendUdpMessage } from "./client.js";
import { settings } from "./const.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const app = express();
const PORT = 3000; // html-server port

// Middleware to parse the text/plain body
app.use(bodyParser.json());

// Serving static files
app.use(express.static(__dirname));

// Route to handle TCP message sending
app.post("/send-tcp-message", (req, res) => {
  const { logFrame, server } = req.body;
  if (!logFrame || !server) {
    return res.status(400).json({ error: "logFrame and server are required" });
  }
  console.log("Received TCP message:", server, logFrame);
  sendTcpMessage(logFrame, server);
  res.status(200).send({ message: "TCP message received" });
});

// Route to handle UDP message sending
app.post("/send-udp-message", (req, res) => {
  const { logFrame, server } = req.body;
  console.log("Received UDP message:", server, logFrame);
  if (!logFrame || !server) {
    return res.status(400).json({ error: "logFrame and server are required" });
  }
  sendUdpMessage(logFrame, server);
  res.status(200).send({ message: "UDP message received" });
});

// Route to serve the index.html file when accessing the URL
app.get("/", (req, res) => {
  res.sendFile(path.join(__dirname, "index.html"));
});

// Start the server
app.listen(PORT, () => {
  console.log(`Server running at http://localhost:${PORT}/ with settings:`);
  console.log(settings);
});
