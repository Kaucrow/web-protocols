import fs from 'fs';
const config = JSON.parse(fs.readFileSync('settings.json', 'utf8'));

export function sendTcpMessage(defaultMessage, cmd, inputId) {
    const inputField = document.getElementById(inputId);
    const userMessage = inputField.value.trim();
    const logMessage = userMessage || defaultMessage; 

    const frame = config.frame;

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

    const frame = config.frame;

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