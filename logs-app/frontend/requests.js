import { settings } from "./const.js";

export function sendTcpMessage(defaultMessage, cmd, inputId) {
    const inputField = document.getElementById(inputId);
    const userMessage = inputField.value.trim();
    const logMessage = userMessage || defaultMessage; 
    console.log('logMessage:',settings);
    const frame = settings.frame;

    const logFrame = [`${frame.init}`, `${cmd}`, `${logMessage}`, `${frame.endData}`, `${frame.close}`].join(frame.delim);

    console.log('Sending message:', logFrame);

    // We send the log frame to the tcp server
    fetch('/send-tcp-message', {
        method: 'POST',
        headers: {
            'Content-Type': 'text/plain'
        },
        body: logFrame 
    })
    .then(response => response.text())
    .then(data => console.log(data))
    .catch(error => console.error('Error:', error));
}

export function sendUdpMessage(defaultMessage, cmd, inputId) {
    const inputField = document.getElementById(inputId);
    const userMessage = inputField.value.trim();
    const logMessage = userMessage || defaultMessage; // Use user input or default message

    const frame = settings.frame;

    const logFrame = [`${frame.init}`, `${cmd}`, `${logMessage}`, `${frame.endData}`, `${frame.close}`].join(frame.delim);

    console.log('Sending message:', logFrame);

    // We send the log frame to the udp server
    fetch('/send-udp-message', {
        method: 'POST',
        headers: {
            'Content-Type': 'text/plain'
        },
        body: logFrame
    })
    .then(response => response.text())
    .then(data => console.log(data))
    .catch(error => console.error('Error:', error));
}

window.sendTcpMessage = sendTcpMessage;
window.sendUdpMessage = sendUdpMessage;