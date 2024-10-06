import net from 'net';
import fs from 'fs';

const server = net.createServer((socket) => {
    const clientAddress = socket.remoteAddress;
    const clientPort = socket.remotePort;

    console.log('Cliente conectado.');

    // Evento cuando el servidor recibe datos del cliente
    socket.on('data', (trama) => {

        //console.log(`Trama de log recibida:${trama}`);
        //console.log('Trama de log recibida:', trama);

        let { fecha, hora } = obtenerFechaYHora();

        let arrayParametros=tramaToArray(trama);

        // Guardar la trama en un archivo de texto
        fs.appendFile('logsServer.txt', ` ${clientAddress}, ${clientPort}, ${arrayParametros[1]}, ${arrayParametros[2]}, ${fecha}:${hora} \n`, (err) => {
            if (err) {
                console.error('Error al escribir en el archivo:', err);
            } else {
                console.log('Log guardado en logs.txt');
            }
        });
    });

    socket.on('end', () => {
        console.log('Cliente desconectado.');
    });
});

// El servidor escucha en el puerto 8080
server.listen(8080, () => {
    console.log('Servidor escuchando en el puerto 8080.');
});

export const tramaToArray= (trama)=>{
    let message=trama.toString();
    let array = message.split("^");
    
    if(array.length!=5){
        console.log('ponga la trama bien coniaso');
        return;
    }
    console.log(array);

    return array;

}

function obtenerFechaYHora() {
    const fecha = new Date();
    const dia = fecha.getDate().toString().padStart(2, '0');
    const mes = (fecha.getMonth() + 1).toString().padStart(2, '0');
    const anio = fecha.getFullYear();
    const horas = fecha.getHours().toString().padStart(2, '0');
    const minutos = fecha.getMinutes().toString().padStart(2, '0');
    const segundos = fecha.getSeconds().toString().padStart(2, '0');
    
    // Devolver fecha y hora por separado
    return {
        fecha: `${dia}/${mes}/${anio}`,
        hora: `${horas}:${minutos}:${segundos}`
    };
}
