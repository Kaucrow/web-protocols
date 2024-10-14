import net from 'net';
import dgram from 'dgram';
import { settings } from './const.js';

const config = settings;
console.log('Settings', config);

export function sendTcpMessage(message, typeServer) {
    console.log('Sending message:', message, typeServer);
    let server;
    if (typeServer === 'node') {
        server = config.node;
    }else if (typeServer === 'rust') {
        sendWsMessage('tcp', logFrame);
    }
    const client = new net.Socket();
    const logFrame = `${message}`;
    console.log('Sending TCP message...', message.server);
    console.log('host:', server.host, 'port:', server.tcp_port);
    console.log('Attempting to connect to TCP server...');
    client.connect(server.tcp_port, server.host, () => {
        console.log('Connected to TCP server');
        client.write(logFrame);
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

export function sendUdpMessage(message, typeServer) {
    console.log('Sending message:', message, typeServer);
    let server;
    if (typeServer === 'node') {
        server = config.node;
    }else if (typeServer === 'rust') {
        server = config.rust;
    }

    const client = dgram.createSocket('udp4');
    const data = Buffer.from(message);

    console.log('host:', server.host, 'port:', server.udp_port);

    console.log('Sending UDP message...');
    client.send(data, server.udp_port, server.host, (error) => {
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
const WS_URL = `ws://${config.rust.host}:${config.rust.tcp_port}/ws`;
const socket = new WebSocket(WS_URL);

socket.addEventListener('open', (event) => {
    console.log('Connected to WebSocket server');
});

function sendWsMessage(cmd, logMessage) {
    if (socket.readyState === WebSocket.OPEN) {
        socket.send(logMessage);
    }
}