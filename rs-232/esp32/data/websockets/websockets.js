const OnLoad = (event) => { InitWebSocket(); }

let gateway = `ws://${window.location.hostname}/ws`;
let websocket;
addEventListener('load', OnLoad);

function InitWebSocket(){
  console.log('INFO: Attempting to open a WebSocket connection...');
  websocket = new WebSocket(gateway);
  websocket.onopen = OnOpen;
  websocket.onclose = OnClose;
  websocket.onmessage = OnMessage;
}

const GetValues = () => { websocket.send("GetValues"); }

function OnOpen(event){
  console.log('INFO: Connection opened.');
  GetValues();
}

function OnClose(event){
  console.log('INFO: Connection closed.');
  setTimeout(InitWebSocket, 2000);
}

function OnMessage(event){
  // Logic for updating the WebSocket clients when receiving a message from the ESP32 server
  console.log(event.data);
}