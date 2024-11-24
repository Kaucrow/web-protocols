import { SerialPort } from 'serialport';
import { ReadlineParser } from '@serialport/parser-readline';
import { handleCommand, parseFrame } from './functions.js';


const port = new SerialPort({ path: 'COM7', baudRate: 9600 });

port.on('open', () => {
    console.log('Server Listening');
    port.write('Server Listening\n');
});

port.on('error', (err) => {
    console.error('Error: ', err.message);
});

const parser = port.pipe(new ReadlineParser({})); 
//console.log('Hola')

parser.on('data', (frame) => {
    //console.log('Received message:', frame);
    if (frame == 'Setup finished.\r'){
        console.log('Connected')
        return
    }
    const parsedFrame = parseFrame(frame);
    
    
    if (parsedFrame) {
        const { command, message } = parsedFrame;
        handleCommand(command, message);
    } else {
        console.log('Invalid frame, skipping command handling.');
    }
});
