function toggleBox(id) {
  const boxes = document.querySelectorAll('.box');
  
  boxes.forEach(box => {
    if (box.id === id) {
      box.classList.toggle('active');
    } else {
      box.classList.remove('active');
    }
  });
}

function sendMessage() {
  let msgInput = document.getElementById('message-input');
  let msg = msgInput.value.trim();

  if (!msg) {
    return;
  }

  if (websocket.readyState === WebSocket.OPEN) {
    websocket.send(JSON.stringify({
      'message': msg
    }));
    console.log(`Sent message: ${msg}`);
  }
}

function createFile() {
  let pathInput = document.getElementById('create-file-path-input');
  let path = pathInput.value.trim();
  
  let contentInput = document.getElementById('create-file-content-input');
  let content = contentInput.value;

  if (!path || !content) {
    return;
  }

  if (websocket.readyState === WebSocket.OPEN) {
    websocket.send(JSON.stringify({
      'path': path,
      'content': content
    }));
    console.log(`Creating file at: '${path}' with content: '${content}'`);
  }
}

function deleteFile() {
  let pathInput = document.getElementById('delete-file-path-input');
  let path = pathInput.value.trim();
  
  if (!path) {
    return;
  }

  if (websocket.readyState === WebSocket.OPEN) {
    websocket.send(JSON.stringify({
      'path': path
    }));
    console.log(`Deleting file at: '${path}'`);
  }
}

function copyFile() {
  let fromPathInput = document.getElementById('copy-file-from-input');
  let fromPath = fromPathInput.value.trim();

  let toPathInput = document.getElementById('copy-file-to-input');
  let toPath = toPathInput.value;

  if (!fromPath || !toPath) {
    return;
  }

  if (websocket.readyState === WebSocket.OPEN) {
    websocket.send(JSON.stringify({
      'fromPath': fromPath,
      'toPath': toPath
    }));
    console.log(`Copying file from: '${fromPath}' to '${toPath}'`);
  }
}