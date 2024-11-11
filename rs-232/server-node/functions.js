import fs from 'fs';
import path from 'path';


//let frame = 'init^send^copy_of_atla.txt^endData^close';  
const __dirname = path.resolve();  

export function parseFrame(frame) {
    // Log the raw frame for debugging
    console.log('Raw frame:', frame);

    
    if (frame.startsWith('init') && frame.endsWith('close\r')) {
        
        let content = frame.slice(4, -14); //antes era -13 
        //console.log('Sliced content:', content);

        
        let parts = content.split('^').filter(part => part.trim() !== '');

        console.log('Split parts:', parts);

        if (parts.length === 2) {
            let command = parts[0];  
            let message = parts[1];  
            
            return { command, message };
        }
    }
    console.log('Invalid frame:', frame);
    return null;  
}


export function handleCommand(command, message) {
    switch (command) {
        case 'create':
            createFile(message);
            break;
        case 'delete':
            deleteFile(message);
            break;
        case 'copy':
            copyFile(message);
            break;
        case 'send':
            receiveMessage(message);
            break;
        default:
            console.log('Unknown command');
    }
}

function createFile(fileName) {
    console.log(fileName)
    try {
        let contentFile = fileName.split('>')
        console.log(contentFile)
        const filePath = path.join(__dirname,  contentFile[1]);
        fs.writeFileSync(filePath, contentFile[0]);  
        console.log(`File created: ${filePath}`);
    } catch (err) {
        console.error('Error creating file:', err);
    }
}


function deleteFile(fileName) {
    try {
        const filePath = path.join(__dirname, fileName);
        if (fs.existsSync(filePath)) {
            fs.unlinkSync(filePath); 
            console.log(`File deleted: ${filePath}`);
        } else {
            console.log('File not found:', filePath);
        }
    } catch (err) {
        console.error('Error deleting file:', err);
    }
}

function copyFile(fileName) {
    try {
        let contentFile = fileName.split('>')
        const sourcePath = path.join(__dirname, contentFile[0]);
        const destinationPath = path.join(__dirname, contentFile[1]);

        if (fs.existsSync(sourcePath)) {
            fs.copyFileSync(sourcePath, destinationPath);  
            console.log(`File copied: ${destinationPath}`);
        } else {
            console.log('Source file not found:', sourcePath);
        }
    } catch (err) {
        console.error('Error copying file:', err);
    }
}


function receiveMessage(message) {
    console.log(`Received message: ${message}`);
}


function simulateUSBFrameProcessing() {
    console.log('Simulating frame processing...');

    
    let parsedFrame = parseFrame(frame);
    if (parsedFrame) {
        console.log(`Parsed Command: ${parsedFrame.command}`);
        console.log(`Parsed Message: ${parsedFrame.message}`);

        
        handleCommand(parsedFrame.command, parsedFrame.message);
    } else {
        console.log('Invalid frame!');
    }
}

// Simulate the USB frame reception
//simulateUSBFrameProcessing();
