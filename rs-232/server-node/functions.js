import fs from 'fs';
import path from 'path';


//let frame = 'init^send^copy_of_atla.txt^endData^close';  
const __dirname = path.resolve();  

export function parseFrame(frame) {
    // Log the raw frame for debugging
    console.log('Raw frame:', frame);

    
    if (frame.startsWith('init') && frame.endsWith('close')) {
        
        let content = frame.slice(4, -13); //antes era -13 
        console.log('Sliced content:', content);

        
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
        case 'receive':
            receiveMessage(message);
            break;
        default:
            console.log('Unknown command');
    }
}

function createFile(fileName) {
    try {
        const filePath = path.join(__dirname, fileName);
        fs.writeFileSync(filePath, 'Faradio y Kaucrow murieron QEPD\n');  
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
        const sourcePath = path.join(__dirname, fileName);
        const destinationPath = path.join(__dirname, 'copy_of_' + fileName);

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
