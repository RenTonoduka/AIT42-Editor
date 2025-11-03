# App Icons

This directory should contain the application icons in various sizes:

- `32x32.png` - 32x32 pixel icon
- `128x128.png` - 128x128 pixel icon
- `128x128@2x.png` - 256x256 pixel icon for Retina displays
- `icon.icns` - macOS icon bundle
- `icon.ico` - Windows icon file

## Generating Icons

You can use the Tauri CLI to generate icons from a single source image:

```bash
cargo tauri icon /path/to/source-icon.png
```

The source image should be at least 512x512 pixels for best results.

## Placeholder Icons

For development, you can use placeholder icons. The build will use default Tauri icons if these files are not present.
