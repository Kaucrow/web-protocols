import dgram from "dgram";
import fs from "fs";
import { getTime, frameToArray, writeLog } from "./utils.js";

const config = JSON.parse(fs.readFileSync("settings.json", "utf8"));
const PORT = config.UDPPORT;
const HOST = config.UDPHOST;

const server = dgram.createSocket("udp4");

server.on("message", (frame, rinfo) => {
  const clientIP = rinfo.address;
  const clientPort = rinfo.port;

  console.log(
    `Message received from ${clientIP}:${clientPort} - content: ${frame}`
  );

  server.send("ok message", clientPort, clientIP, (err) => {
    if (err) {
      console.log(err);
    } else {
      console.log("Delivered to the client");
    }
  });

  let { date, time } = getTime();
  let arrayParams = frameToArray(frame);

  let typeMessage = arrayParams[1];
  let message = arrayParams[2];
  //time | IP:port | type | message |
  writeLog(
    `|${date}T${time}|${clientIP}:${clientPort}|${typeMessage}|${message}|  udp   |\n`
  );
});
server.on("listening", () => {
  const address = server.address();
  console.log(`UDP server is running on: ${address.address}:${address.port}`);
});

server.on("error", (err) => {
  console.log("error:" + err);
  server.close();
});

server.bind(PORT, HOST);
