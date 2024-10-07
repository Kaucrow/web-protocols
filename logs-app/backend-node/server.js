import net from 'net';
import fs from 'fs';
import { obtenerFechaYHora,tramaToArray,writeLog} from './utils.js';

const config = JSON.parse(fs.readFileSync('settings.json', 'utf8'));
const PORT = config.PORT;
const HOST = config.HOST;

const server = net.createServer((socket) => {
    const clientAddress = socket.remoteAddress;
    const clientPort = socket.remotePort;

    console.log('Cliente conectado.');

    // Evento cuando el servidor recibe datos del cliente
    socket.on('data', (trama) => {
        let { fecha, hora } = obtenerFechaYHora();
        let arrayParametros=tramaToArray(trama);
        let tipoMensaje = arrayParametros[1]
        let mensaje = arrayParametros[2]
        //hora | IP:puerto |  tipo | mensaje | 
        writeLog(`|${fecha}T${hora}|${clientAddress}:${clientPort}|${tipoMensaje}|${mensaje}|\n`);
    });

    
    socket.on("close",(hadError)=>{
        console.log("Cliente:",clientAddress,"port:", clientPort, "desconectado.")
    })
});


server.listen(PORT,HOST, () => {
    console.log(`Servidor escuchando en el puerto ${PORT}.`);
});




