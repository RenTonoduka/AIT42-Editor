#!/usr/bin/env node
/**
 * Convert SVG icon to PNG using browser-based rendering
 * Works without external dependencies
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// For Node.js environments without canvas, we'll use a simple approach
// that creates a data URL and writes instructions

const iconDir = path.join(__dirname, '..', 'src-tauri', 'icons');
const svgPath = path.join(iconDir, 'icon.svg');
const pngPath = path.join(iconDir, 'icon.png');

console.log('Converting SVG to PNG...');
console.log('SVG path:', svgPath);

// Read SVG
const svgContent = fs.readFileSync(svgPath, 'utf8');

// Create an HTML file that can be opened in a browser to generate PNG
const htmlContent = `<!DOCTYPE html>
<html>
<head>
    <title>Icon Converter</title>
    <style>
        body {
            margin: 0;
            padding: 20px;
            font-family: system-ui;
            background: #f5f5f5;
        }
        #container {
            text-align: center;
            max-width: 1200px;
            margin: 0 auto;
        }
        canvas {
            border: 2px solid #ccc;
            margin: 20px auto;
            display: block;
            background: white;
        }
        button {
            background: #667eea;
            color: white;
            border: none;
            padding: 12px 24px;
            font-size: 16px;
            border-radius: 6px;
            cursor: pointer;
            margin: 10px;
        }
        button:hover {
            background: #5568d3;
        }
        #instructions {
            background: white;
            padding: 20px;
            border-radius: 8px;
            margin-top: 20px;
            text-align: left;
        }
    </style>
</head>
<body>
    <div id="container">
        <h1>AIT42-Editor Icon Generator</h1>
        <canvas id="canvas" width="1024" height="1024"></canvas>
        <div>
            <button onclick="downloadPNG()">Download PNG (1024x1024)</button>
            <button onclick="downloadSmall()">Download PNG (512x512)</button>
        </div>
        <div id="instructions">
            <h2>Instructions:</h2>
            <ol>
                <li>Click "Download PNG (1024x1024)" to save the icon</li>
                <li>Save it as <code>icon.png</code> in <code>src-tauri/icons/</code></li>
                <li>Run: <code>npm run tauri icon src-tauri/icons/icon.png</code></li>
            </ol>
        </div>
    </div>

    <script>
        const svgData = \`${svgContent}\`;

        function renderSVG(size = 1024) {
            const canvas = document.getElementById('canvas');
            canvas.width = size;
            canvas.height = size;
            const ctx = canvas.getContext('2d');

            const img = new Image();
            const svgBlob = new Blob([svgData], { type: 'image/svg+xml' });
            const url = URL.createObjectURL(svgBlob);

            img.onload = function() {
                ctx.clearRect(0, 0, size, size);
                ctx.drawImage(img, 0, 0, size, size);
                URL.revokeObjectURL(url);
            };

            img.src = url;
            return canvas;
        }

        function downloadPNG() {
            const canvas = renderSVG(1024);
            setTimeout(() => {
                canvas.toBlob(function(blob) {
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = 'icon.png';
                    a.click();
                    URL.revokeObjectURL(url);
                });
            }, 100);
        }

        function downloadSmall() {
            const tempCanvas = document.createElement('canvas');
            tempCanvas.width = 512;
            tempCanvas.height = 512;
            const ctx = tempCanvas.getContext('2d');

            const img = new Image();
            const svgBlob = new Blob([svgData], { type: 'image/svg+xml' });
            const url = URL.createObjectURL(svgBlob);

            img.onload = function() {
                ctx.drawImage(img, 0, 0, 512, 512);
                tempCanvas.toBlob(function(blob) {
                    const downloadUrl = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = downloadUrl;
                    a.download = 'icon-512.png';
                    a.click();
                    URL.revokeObjectURL(downloadUrl);
                    URL.revokeObjectURL(url);
                });
            };

            img.src = url;
        }

        // Auto-render on load
        window.onload = () => renderSVG(1024);
    </script>
</body>
</html>`;

// Write HTML converter
const htmlPath = path.join(iconDir, 'icon-converter.html');
fs.writeFileSync(htmlPath, htmlContent);

console.log('\n✓ Created icon converter HTML file');
console.log('\nTo generate PNG icon:');
console.log('1. Open in browser:', htmlPath);
console.log('2. Click "Download PNG (1024x1024)"');
console.log('3. Save as icon.png in src-tauri/icons/');
console.log('4. Run: npm run tauri icon src-tauri/icons/icon.png');
console.log('\nOr use online converter:');
console.log('- https://cloudconvert.com/svg-to-png');
console.log('- Upload:', svgPath);
console.log('- Set size: 1024x1024');
console.log('- Download and save as icon.png');
