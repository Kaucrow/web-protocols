import net from 'net';
import { obtenerFechaYHora,tramaToArray,writeLog} from './utils.js';

const PORT = 8080;

const server = net.createServer((socket) => {
    const clientAddress = socket.remoteAddress.split('::ffff:')[1];

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

// El servidor escucha en el puerto 8080
server.listen(PORT, () => {
    console.log(`Servidor escuchando en el puerto ${PORT}.`);
});




