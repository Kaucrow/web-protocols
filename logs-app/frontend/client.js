import net from 'net';
import dgram from 'dgram';
import fs from "fs";


const server = JSON.parse(fs.readFileSync('settings.json', 'utf8'));


export function sendTcpMessage(message) {
    const client = new net.Socket();
    const logFrame = `${message}`;

    console.log('Setting:', server);
    console.log('Attempting to connect to TCP server...');
    client.connect(server.node.tcp_port, server.node.host, () => {
        console.log('Connected to TCP server');
        client.write(logFrame);
        sendWsMessage('tcp', logFrame);
    });

    client.on('data', (data) => {
        console.log('Message received from TCP server: ' + data);
        client.destroy();
    });

    client.on('error', (err) => {
        console.error('Connection error: ' + err);
    });

    client.on('close', () => {
        console.log('Connection closed');
    });
}

export function sendUdpMessage(message) {
    const client = dgram.createSocket('udp4');
    const data = Buffer.from(message);

    console.log('Sending UDP message...');
    client.send(data, server.node.udp_port, server.node.host, (error) => {
        if (error) {
            console.error('Error sending UDP message:', error);
        } else {
            console.log('UDP message sent');
        }
    });

    client.on('message', (msg, info) => {
        console.log('Data received from UDP server: ' + msg.toString());
        console.log(`Bytes received: ${msg.length} from ${info.address}:${info.port}`);
    });
}

// WebSocket (Rust) connection
const WS_URL = `ws://${server.rust.host}:${server.rust.tcp_port}/ws`;
const socket = new WebSocket(WS_URL);

socket.addEventListener('open', (event) => {
    console.log('Connected to WebSocket server');
});

function sendWsMessage(cmd, logMessage) {
    if (socket.readyState === WebSocket.OPEN) {
        socket.send(logMessage);
    }
}