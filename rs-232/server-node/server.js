import { SerialPort } from 'serialport';
import { ReadlineParser } from '@serialport/parser-readline';
import fs from 'fs';
import path from 'path';

const port = new SerialPort({ path: 'COM4', baudRate: 9600 });

port.on('open', () => {
    console.log('Server Listening');
    port.write('Server Listening\n');
});

port.on('error', (err) => {
    console.error('Error: ', err.message);
});

const parser = port.pipe(new ReadlineParser({ delimiter: '\r\n' }));

parser.on('data', (data) => {
    console.log(`Received message: ${data}`);
});