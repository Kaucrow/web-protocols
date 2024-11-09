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
    console.log(event.data);
    /* CURRENTLY NOT IMPLEMENTED:
    let myObj = JSON.parse(event.data);
    let keys = Object.keys(myObj);*/

    /*for(let i = 0; i < keys.length; i++){
        let key = keys[i];
        document.getElementById(key).innerHTML = myObj[key];
    }*/
}

//================================
// APPLICATION-SPECIFIC FUNCTIONS
//================================
const TurnOn = () => { websocket.send("ledon"); }
const TurnOff = () => { websocket.send("ledoff"); }
const BtnClick = () => { websocket.send("Hello world from WebSocket!"); }