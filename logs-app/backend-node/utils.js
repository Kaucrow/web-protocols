import fs from 'fs';

export const tramaToArray= (trama)=>{
  let message=trama.toString();
  let array = message.split("^");
  
  if(array.length!=5){
      console.log('error al processar la trama:',array);
      return;
  }
  console.log(array);

  return array;

}

export const obtenerFechaYHora=()=> {
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

export const writeLog = (log) => {
  fs.appendFile('logsServer.txt', log, (err) => {
      if (err) {
          console.error('Error al escribir en el archivo:', err);
      }
  });
}