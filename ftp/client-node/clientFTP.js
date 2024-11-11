import ftp from "ftp";
import inquirer from "inquirer";
import fs from "fs";
import path from "path";

const client = new ftp();

async function configureClient() {
  /*   let connected = false;

  while (!connected) {
    const answers = await inquirer.prompt([
      { type: "input", name: "host", message: "FTP Host:" },
      { type: "input", name: "user", message: "FTP Username:" },
      { type: "password", name: "password", message: "FTP Password:" },
      {
        type: "confirm",
        name: "passive",
        message: "Use passive mode?",
        default: true,
      },
    ]);

    try {
      await connectToFtp(answers);
      connected = true;
    } catch (err) {
      console.error("Error connecting to the FTP server:", err);
      const retryAnswer = await inquirer.prompt([
        {
          type: "confirm",
          name: "retry",
          message: "Do you want to try again?",
          default: true,
        },
      ]);
      if (!retryAnswer.retry) {
        break;
      }
    }
  }

  if (connected) {
    await showMenu();
  } */

  client.connect({
    host: "localhost",
    user: "ats",
    password: "",
    passive: true,
  });

  client.on("ready", () => {
    console.log("Connected to FTP server");
    showMenu();
  });
}

/* function connectToFtp(answers) {
  return new Promise((resolve, reject) => {
    client.on("ready", resolve);
    client.on("error", reject);

    client.connect({
      host: answers.host,
      user: answers.user,
      password: answers.password,
      passive: answers.passive,
    });
  });
} */

async function showMenu() {
  const answers = await inquirer.prompt([
    {
      type: "list",
      name: "action",
      message: "Choose an action:",
      choices: ["Send", "Receive"],
    },
  ]);

  if (answers.action === "Send") {
    await sendFile();
  } else if (answers.action === "Receive") {
    await receiveFile();
  }
}

async function sendFile() {
  try {
    const localPath = await localListFiles();
    const remotePath = await listFiles(
      "Select the remote directory to upload the file:",
      "/",
      "create"
    );

    console.log("Uploading file...", localPath);
    console.log("Saving to...", remotePath);

    client.put(localPath, remotePath, (err) => {
      if (err) {
        throw err;
      }
      console.log("File uploaded successfully");
      client.end();
    });
  } catch (err) {
    console.error("Error uploading the file:", err);
    client.end();
  }
}

async function receiveFile() {
  try {
    const remoteFile = await listFiles("Select the remote file to download:");

    const newLocalPath = await localListFiles(process.cwd(), "create");

    console.log("Saving to...", newLocalPath);
    console.log("Downloading file...", remoteFile);

    client.get(remoteFile, (err, stream) => {
      if (err) {
        if (err.code === 551) {
          console.error("Error: The specified remote file does not exist.");
        } else {
          console.error("Error downloading the file:", err);
        }
        client.end();
        return;
      }

      stream.once("close", () => {
        console.log("File downloaded successfully");
        client.end();
      });

      stream.pipe(fs.createWriteStream(newLocalPath));
    });

    client.on("end", () => {
      console.log("Connection closed");
    });
  } catch (err) {
    console.error("Error downloading the file:", err);
    client.end();
  }
}

async function listFiles(message, currentPath = "/", type) {
  return new Promise((resolve, reject) => {
    client.list(currentPath, async (err, list) => {
      if (err) {
        return reject(err);
      }

      let choices;
      if (type === "create") {
        choices = [
          { name: ".. (parent directory)", value: "..", short: ".." }, // Add option to go to parent
          { name: "Create new file", value: "create", short: "create" },
          ...list.map((item) => {
            const type = item.type === "d" ? "directory" : "file";
            return {
              name: `${item.name} (${type})`,
              value: { name: item.name, type: item.type },
              short: item.name,
            };
          }),
        ];
      } else {
        choices = [
          { name: ".. (parent directory)", value: "..", short: ".." }, // Add option to go to parent
          ...list.map((item) => {
            const type = item.type === "d" ? "directory" : "file";
            return {
              name: `${item.name} (${type})`,
              value: { name: item.name, type: item.type },
              short: item.name,
            };
          }),
        ];
      }

      try {
        const answers = await inquirer.prompt([
          {
            type: "list",
            name: "file",
            message,
            choices,
          },
        ]);

        if (answers.file === "..") {

          currentPath = path.posix.dirname(currentPath); // Go to parent directory
          return resolve(await changeWorkingDirectory(currentPath, type));
        }
        if (answers.file.type === "d") {// Go to selected directory
          currentPath = path.posix.join(currentPath, answers.file.name);
          return resolve(await changeWorkingDirectory(currentPath, type));
        }
        if (answers.file === "create") {
          //User wants to create a new file
          const newArchive = await inquirer.prompt([
            {
              type: "input",
              name: "newArchive",
              message: "Enter the name of the new archive:",
            },
          ]);

          currentPath = path.posix.join(currentPath, newArchive.newArchive);
          resolve(currentPath);
          return;
        }


        currentPath = path.posix.join(currentPath, answers.file.name);
        currentPath = currentPath.substring(1);
        console.log("new currentPath listFile", currentPath);
        

        return resolve(answers.file.name);
      } catch (err) {
        reject(err);
      }
    });
  });
}

async function changeWorkingDirectory(cwd, type) {
  return new Promise((resolve, reject) => {
    client.cwd(cwd, (err, currentDir) => {
      if (err) {
        console.error("Error changing directory:", err);
        return reject(err);
      }
      client.pwd((err, currentDir) => {
        if (err) {
          console.error("Error getting current directory:", err);
          return reject(err);
        }
        resolve(listFiles("Select remote file:", currentDir, type));
      });
    });
  });
}

async function localListFiles(currentPath = process.cwd(), type) {
  return new Promise((resolve, reject) => {
    fs.readdir(currentPath, { withFileTypes: true }, async (err, files) => {
      if (err) {
        return reject(err);
      }
      let choices;

      if (type === "create") {
        choices = [
          { name: ".. (parent directory)", value: "..", short: ".." }, // Add option to go to parent directory
          { name: "Create new file", value: "create", short: "create" },
          ...files.map((file) => {
            const type = file.isDirectory() ? "directory" : "file";
            return {
              name: `${file.name} (${type})`,
              value: { name: file.name, type },
              short: file.name,
            };
          }),
        ];
      } else {
        choices = [
          { name: ".. (parent directory)", value: "..", short: ".." }, // Add option to go to parent directory
          ...files.map((file) => {
            const type = file.isDirectory() ? "directory" : "file";
            return {
              name: `${file.name} (${type})`,
              value: { name: file.name, type },
              short: file.name,
            };
          }),
        ];
      }

      try {
        const answers = await inquirer.prompt([
          {
            type: "list",
            name: "file",
            message: "Select a file or directory:",
            choices,
          },
        ]);

        if (answers.file === "..") {
          currentPath = path.resolve(currentPath, ".."); // Go to parent directory
          resolve(await localListFiles(currentPath, type));
        }

        if (answers.file.type === "directory") {
          currentPath = path.join(currentPath, answers.file.name);
          resolve(await localListFiles(currentPath, type));
        }

        if (answers.file === "create") {
          const newDir = await inquirer.prompt([
            {
              type: "input",
              name: "name",
              message: "Enter the name of the new file:",
            },
          ]);

          resolve(path.join(currentPath, newDir.name));
        }

        resolve(path.join(currentPath, answers.file.name));
      } catch (err) {
        reject(err);
      }
    });
  });
}

configureClient();
