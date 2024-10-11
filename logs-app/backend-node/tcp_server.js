import net from 'net';
import fs from 'fs';
import { getTime,frameToArray,writeLog} from './utils.js';

const config = JSON.parse(fs.readFileSync('settings.json', 'utf8'));
const PORT = config.TCPPORT;
const HOST = config.TCPHOST;

const server = net.createServer((socket) => {
    const clientIP = socket.remoteAddress;
    const clientPort = socket.remotePort;

    console.log('Client connected.');

    // Evento cuando el servidor recibe datos del cliente
    socket.on('data', (trama) => {
        let { date, time } = getTime();
        let arrayParams=frameToArray(trama);
        let typeMessage = arrayParams[1]
        let message = arrayParams[2]
        //hora | IP:puerto |  tipo | mensaje | 
        writeLog(`|${date}T${time}|${clientIP}:${clientPort}|${typeMessage}|${message}|  tcp   |\n`);
    });

    
    socket.on("close",(hadError)=>{
        console.log("Client:",clientIP,"port:", clientPort, "disconnected.")
    })

    socket.on("error",(err)=>{
        console.log("Error:",err.message);
    })
});

// El servidor escucha en el puerto 8080
server.listen(PORT,HOST, () => {
    console.log(`Server listening on port ${PORT}.`);
});




