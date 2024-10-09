import fs from 'fs';

export const tramaToArray= (trama)=>{
  let message=trama.toString();
  let array = message.split("^");
  
  if(array.length!=5){
      console.log('Error processing:',array);
      return;
  }
  console.log(array);

  return array;

}

export const getTime=()=> {
  const date = new Date();
  const day = date.getDate().toString().padStart(2, '0');
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const year = date.getFullYear();
  const hour = date.getHours().toString().padStart(2, '0');
  const minutes = date.getMinutes().toString().padStart(2, '0');
  const seconds = date.getSeconds().toString().padStart(2, '0');
  
  // Devolver fecha y hora por separado
  return {
      date: `${day}/${month}/${year}`,
      time: `${hour}:${minutes}:${seconds}`
  };
}

export const writeLog = (log) => {
  fs.appendFile('logsServer.txt', log, (err) => {
      if (err) {
          console.error('Error writing on the file:', err);
      }
  });
}