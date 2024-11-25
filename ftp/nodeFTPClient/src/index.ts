import express from "express";
import ftp from "ftp";
import cors from "cors";
import fs from "fs";
import { formatResponse } from "./format";

const serverHTTP = express();
const port = 3040;
const startPath = "./Atlas";

serverHTTP.use(cors());
serverHTTP.use(express.json()); // Add this line to parse JSON bodies

const clientFTP = new ftp();

clientFTP.on("end", () => {
  console.log("Disconnected from FTP server");
});

serverHTTP.use((req, res, next) => {
  console.log(`Route: ${req.path}, Method: ${req.method}`);
  if (Object.keys(req.body).length) {
    console.log("Body:", req.body);
  }
  next();
});

serverHTTP.post("/api/connect", (req, res) => {
  const { host, user, password } = req.body;
  console.log("Request body:", req.body);

  if (!host || !user || !password) {
    console.log({message: "Missing required fields"});
    return res
    
      .status(400)
      .send({ success: false, message: "Missing required fields" });
  }

  let responseSent = false;

  clientFTP.connect({ host, user, password });

  clientFTP.on("ready", () => {
    if (!responseSent) {
      responseSent = true;
      console.log({message: "Connected to FTP server"});
      return res.send({ success: true, message: "Connected to FTP server" });
    }
  });

  clientFTP.on("error", (err) => {
    if (!responseSent) {
      responseSent = true;
      console.log({message: "Error connecting to FTP server"});
      return res
        .status(500)
        .send({ success: false, message: "Error connecting to FTP server" });
    }
  });
});

serverHTTP.get("/api/disconnect", (req, res) => {
  if (clientFTP.connected) {
    clientFTP.end();
    console.log({message: "Disconnected from FTP server"});
    return res.send({ success: true, message: "Disconnected from FTP server" });
  } else {
    console.log({message: "Not connected to any FTP server"});
    return res
      .status(400)
      .send({ success: false, message: "Not connected to any FTP server" });
  }
});

serverHTTP.get("/api/get-server-files", (req, res) => {
  if (!clientFTP.connected) {
    console.log({message: "Not connected to any FTP server"});
    return res
      .status(400)
      .send({ success: false, message: "Not connected to any FTP server" });
  }

  clientFTP.list((err, list) => {
    if (err) {
      console.log({message: "Error retrieving file list from FTP server"});
      return res.status(500).send({
        success: false,
        message: "Error retrieving file list from FTP server",
      });
    }
    const formattedFile = formatResponse(list);

    console.log({message: "Files from FTP server"});
    return res.send({
      success: true,
      files: formattedFile,
      messafe: "Files from FTP server",
    });
  });
});

serverHTTP.get("/api/get-client-files", (req, res) => {
  fs.readdir(startPath, { withFileTypes: true }, (err, files) => {
    if (err) {
      console.log({message: "Error retrieving file list from client directory"});
      return res.status(500).send({
        success: false,
        message: "Error retrieving file list from client directory",
      });
    }

    const formattedFiles = files.map((file: any) => {
      const filePath = `${startPath}/${file.name}`;
      const stats = fs.statSync(filePath);
      return {
        name: file.name,
        type: file.isDirectory() ? "directory" : "file",
        size: `${(stats.size / 1024).toFixed(1)} KB`,
        date: stats.mtime.toISOString().split("T")[0],
      };
    });
    console.log({message: "send Files from client directory"});
    return res.send({ success: true, files: formattedFiles });
  });
});

serverHTTP.post("/api/download-server-file", (req, res) => {
  try {
    const { path } = req.body;

    if (!path) {
      console.log({message: "Missing file path"});
      return res
        .status(400)
        .send({ success: false, message: "Missing file path" });
    }
    console.log("path:", path);
    console.log("connect:", clientFTP.connected);

    if (!clientFTP.connected) {
      console.log({message: "Not connected to any FTP server"});
      return res
        .status(400)
        .send({ success: false, message: "Not connected to any FTP server" });
    }

    clientFTP.get(`${path}`, (err, stream) => {
      if (err) {
        console.log({message: "Error downloading file from FTP server"});
        return res.status(500).send({
          success: false,
          message: "Error downloading file from FTP server",
        });
      }

      const localFilePath = `${startPath}/${path.split("/").pop()}`;
      const writeStream = fs.createWriteStream(localFilePath);

      stream.pipe(writeStream);

      stream.on("end", () => {
        console.log({message: "File downloaded successfully"});
        res.send({ success: true, message: "File downloaded successfully" });
      });

      stream.on("error", (err) => {
        console.log({message: "Error reading file stream"});
        res
          .status(500)
          .send({ success: false, message: "Error reading file stream" });
      });
    });
  } catch (e) {
    console.error(e);
    console.log({message: "Error downloading file from FTP server"});
    res.status(500).send({
      success: false,
      message: "Error downloading file from FTP server",
    });
  }
});

serverHTTP.post("/api/send-file", (req, res) => {
  const { path } = req.body;

  if (!path) {
    console.log({message: "Missing file path"});
    return res
      .status(400)
      .send({ success: false, message: "Missing file path" });
  }

  if (!clientFTP.connected) {
    console.log({message: "Not connected to any FTP server"});
    return res
      .status(400)
      .send({ success: false, message: "Not connected to any FTP server" });
  }

  const localFilePath = `${startPath}/${path}`;
  const fileStream = fs.createReadStream(localFilePath);

  let responseSent = false;

  fileStream.on("error", (err) => {
    if (!responseSent) {
      responseSent = true;
      if (err.code === "ENOENT") {
        console.log({message: "File not found on client"});
        return res.status(404).send({
          success: false,
          message: "File not found on client",
        });
      } else {
        console.log({message: "Error reading file from client"});
        return res.status(500).send({
          success: false,
          message: "Error reading file from client",
        });
      }
    }
  });

  clientFTP.put(fileStream, path, (err) => {
    if (!responseSent) {
      responseSent = true;
      if (err) {
        console.error(err);
        console.log({message: "Error uploading file to FTP server"});
        return res.status(500).send({
          success: false,
          message: "Error uploading file to FTP server",
        });
      }
      console.log({message: "File uploaded successfully"});
      return res.send({ success: true, message: "File uploaded successfully" });
    }
  });
});

serverHTTP.post("/api/change-directory", (req, res) => {
  const { path } = req.body;
  console.log("path:", path);

  if (!path) {
    console.log({message: "Missing directory path"});
    return res
      .status(400)
      .send({ success: false, message: "Missing directory path" });
  }

  if (!clientFTP.connected) {
    console.log({message: "Not connected to any FTP server"});
    return res
      .status(400)
      .send({ success: false, message: "Not connected to any FTP server" });
  }

  clientFTP.cwd(path, (err) => {
    if (err) {
      console.log({message: "Error changing directory on FTP server"});
      return res.status(500).send({
        success: false,
        message: "Error changing directory on FTP server",
      });
    }

    return res.send({
      success: true,
      message: "Directory changed successfully",
    });
  });
});

serverHTTP.post("/api/delete-file", (req, res) => {
  const { path } = req.body;

  if (!path) {
    console.log({message: "Missing file path"});
    return res
      .status(400)
      .send({ success: false, message: "Missing file path" });
  }

  if (!clientFTP.connected) {
    console.log({message: "Not connected to any FTP server"});
    return res
      .status(400)
      .send({ success: false, message: "Not connected to any FTP server" });
  }

  clientFTP.delete(path, (err) => {
    if (err) {
      console.log({message: "Error deleting file on FTP server"});
      return res.status(500).send({
        success: false,
        message: "Error deleting file on FTP server",
      });
    }
    console.log({message: "File deleted successfully"});
      return res.send({ success: true, message: "File deleted successfully" });
  });
});

serverHTTP.use((req, res, next) => {
  console.log({message: "Route not found"});
  res.status(404).send({ success: false, message: "Route not found" });
});

serverHTTP.listen(port, () => {
  console.log(`Server running at http://localhost:${port}`);
});
