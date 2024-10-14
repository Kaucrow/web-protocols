// Application-wide settings
export const settings = {};

import fs from 'fs';
settings = JSON.parse(fs.readFileSync('settings.json', 'utf8'));
