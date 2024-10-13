import express from 'express';
import bodyParser from 'body-parser';
import path from 'path';
import { fileURLToPath } from 'url';
import { sendTCPMessage, sendUDPMessage } from './client.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const app = express();
const PORT = 3000;  //this is the port for the html-server

// Middleware to parse the text/plain body
app.use(bodyParser.text());

// Serving static files 
app.use(express.static(path.join(__dirname, 'frontend')));

// Route to handle TCP message sending
app.post('/send-tcp-message', (req, res) => {
    const message = req.body;
    //const cmd = message.split('^')[2]
    console.log('Received TCP message:', message);
    sendTCPMessage(message);
    res.status(200).send('TCP message received');
});

// Route to handle UDP message sending
app.post('/send-udp-message', (req, res) => {
    const message = req.body;
    console.log('Received UDP message:', message);
    sendUDPMessage(message);
    res.status(200).send('UDP message received');
});

// Route to serve the index.html file when accessing the URL
app.get('/', (req, res) => {
    res.sendFile(path.join(__dirname, 'index.html'));
});

// Start the server
app.listen(PORT, () => {
    console.log(`Server running at http://localhost:${PORT}/`);
});
