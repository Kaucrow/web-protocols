// Application-wide settings
<<<<<<< HEAD
export const settings = {};

import fs from 'fs';
settings = JSON.parse(fs.readFileSync('settings.json', 'utf8'));
=======
export let settings = {};

// If running on the browser
if (typeof window !== 'undefined') {
    try {
        const res = await fetch('./settings.json');
        if (!res.ok) {
            throw new Error(res);
        }
        const data = await res.json();

        settings = {
            server: data.rust,
            frame: data.frame,
        };
    } catch (err) {
        console.error('Error loading settings:', err);
    }
// If running on Node
} else {
    const fs = await import('fs');
    const path = await import('path');
    const { fileURLToPath } = await import('url');

    const __filename = fileURLToPath(import.meta.url);
    const __dirname = path.dirname(__filename);

    const settingsPath = path.join(__dirname, 'settings.json');

    if (fs.existsSync(settingsPath)) {
        settings = JSON.parse(fs.readFileSync(settingsPath, 'utf-8'));
        settings = {
            server: settings.rust,
            frame: settings.frame
        };
    }
}
>>>>>>> parent of 2ee0a72 (fix: correct issue with node host and port)
