import net from 'net';
import dgram from 'dgram';

//We change this depending of the port and Host being used in the TCP server
const TCP_PORT = 8080;
const TCP_HOST = '0.0.0.0';

export function sendTCPMessage(message) {
    const client = new net.Socket();

    console.log('Attempting to connect to TCP server...');
    client.connect(TCP_PORT, TCP_HOST, () => {
        console.log('Connected to TCP server');
        client.write(message);
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

//We change this depending of the port and Host being used in the UDP server
const UDP_PORT = 6666;
const UDP_HOST = '0.0.0.0';

export function sendUDPMessage(message) {
    const client = dgram.createSocket('udp4');
    const data = Buffer.from(message);

    console.log('Sending UDP message...');
    client.send(data, UDP_PORT, UDP_HOST, (error) => {
        if (error) {
            console.error('Error sending UDP message:', error);
            client.close();
        } else {
            console.log('UDP message sent');
        }
    });

    client.on('message', (msg, info) => {
        console.log('Data received from UDP server: ' + msg.toString());
        console.log(`Bytes received: ${msg.length} from ${info.address}:${info.port}`);
    });
}
