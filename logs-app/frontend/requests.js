import { settings } from "./const.js";

export function sendTcpMessage(defaultMessage, cmd, inputId) {
  const inputField = document.getElementById(inputId);
  const userMessage = inputField.value.trim();
  const logMessage = userMessage || defaultMessage;

  const checkbox = document.getElementById("toggle-checkbox");
  let typeServer = checkbox.checked ? "node" : "rust";

  const frame = settings.frame;

  const logFrame = [
    `${frame.init}`,
    `${cmd}`,
    `${logMessage}`,
    `${frame.endData}`,
    `${frame.close}`,
  ].join(frame.delim);

  const data = {
    logFrame: logFrame,
    server: typeServer,
  };
  console.log("Data:", data);

  // We send the log frame to the tcp server
  fetch("/send-tcp-message", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  })
    .then((response) => response.json())
    .then((data) => {
      console.log("Success:", data);
    })
    .catch((error) => {
      console.error("Error:", error);
    });
}

export function sendUdpMessage(defaultMessage, cmd, inputId) {
  const inputField = document.getElementById(inputId);
  const userMessage = inputField.value.trim();
  const logMessage = userMessage || defaultMessage; // Use user input or default message

  const checkbox = document.getElementById("toggle-checkbox");
  let typeServer = checkbox.checked ? "node" : "rust";

  const frame = settings.frame;

  const logFrame = [
    `${frame.init}`,
    `${cmd}`,
    `${logMessage}`,
    `${frame.endData}`,
    `${frame.close}`,
  ].join(frame.delim);

  console.log("Sending message:", logFrame);

  const data = {
    logFrame: logFrame,
    server: typeServer,
  };

  // We send the log frame to the udp server
  fetch("/send-udp-message", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  })
    .then((response) => response.json())
    .then((data) => {
      console.log("Success:", data);
    })
    .catch((error) => {
      console.error("Error:", error);
    });
}

window.sendTcpMessage = sendTcpMessage;
window.sendUdpMessage = sendUdpMessage;
