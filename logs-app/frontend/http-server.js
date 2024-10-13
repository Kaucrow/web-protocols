import express from 'express';
import bodyParser from 'body-parser';
import path from 'path';
import { fileURLToPath } from 'url';
import { sendTcpMessage, sendUdpMessage } from './client.js';
import fs from 'fs';

const settings = JSON.parse(fs.readFileSync('settings.json', 'utf8'));

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const app = express();
const PORT = 3000;  // html-server port

// Middleware to parse the text/plain body
app.use(bodyParser.text());

// Serving static files 
app.use(express.static(__dirname));

// Route to handle TCP message sending
app.post('/send-tcp-message', (req, res) => {
    const message = req.body;
    //const cmd = message.split('^')[2]
    console.log('Received TCP message:', message);
    sendTcpMessage(message);
    res.status(200).send('TCP message received');
});

// Route to handle UDP message sending
app.post('/send-udp-message', (req, res) => {
    const message = req.body;
    console.log('Received UDP message:', message);
    sendUdpMessage(message);
    res.status(200).send('UDP message received');
});

// Route to serve the index.html file when accessing the URL
app.get('/', (req, res) => {
    res.sendFile(path.join(__dirname, 'index.html'));
});

// Start the server
app.listen(PORT, () => {
    console.log(`Server running at http://localhost:${PORT}/ with settings:`);
    console.log("node:",settings.node);
    console.log("rust:",settings.rust);
    console.log("frame:",settings.frame);
});