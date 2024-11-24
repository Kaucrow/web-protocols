import { SerialPort } from "serialport";
import { ReadlineParser } from "@serialport/parser-readline";

// Configure the serial port
const port = new SerialPort({
  path: "COM4",
  baudRate: 9600,
});

// Set up the parser
const parser = port.pipe(new ReadlineParser({ delimiter: "^" }));

// Open the port
port.on("open", () => {
  console.log("Serial Port Opened");
});

// Add error handling for the port
port.on("error", (err) => {
  console.error("Error opening the port: ", err.message);
});

// Send a message
function sendMessage(message) {
  port.write(message + "\n", (err) => {
    if (err) {
      return console.log("Error on write: ", err.message);
    }
    console.log("Message sent: ", message);
  });
}

// Example usage
sendMessage("init^create^server.txt^endData^close");

// Listen for incoming data
parser.on("data", (data) => {
  console.log("Received: ", data);
});
