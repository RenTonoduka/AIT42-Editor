#!/usr/bin/env python3
"""
Generate AIT42-Editor application icon
Creates a 1024x1024 PNG icon with a modern design
"""

def generate_icon_svg():
    """Generate SVG icon that can be converted to PNG"""
    svg = '''<?xml version="1.0" encoding="UTF-8"?>
<svg width="1024" height="1024" viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg">
  <!-- Background gradient -->
  <defs>
    <linearGradient id="bgGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#667eea;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#764ba2;stop-opacity:1" />
    </linearGradient>
    <linearGradient id="iconGrad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#fbbf24;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#f59e0b;stop-opacity:1" />
    </linearGradient>
  </defs>

  <!-- Background circle -->
  <circle cx="512" cy="512" r="480" fill="url(#bgGrad)"/>

  <!-- Code editor icon -->
  <g transform="translate(512, 512)">
    <!-- Bracket symbols representing code -->
    <path d="M -200,-200 L -150,-200 L -150,-150 M -200,200 L -150,200 L -150,150"
          stroke="url(#iconGrad)" stroke-width="40" fill="none" stroke-linecap="round"/>
    <path d="M 200,-200 L 150,-200 L 150,-150 M 200,200 L 150,200 L 150,150"
          stroke="url(#iconGrad)" stroke-width="40" fill="none" stroke-linecap="round"/>

    <!-- AI brain symbol in center -->
    <circle cx="0" cy="-80" r="30" fill="url(#iconGrad)"/>
    <circle cx="-60" cy="0" r="30" fill="url(#iconGrad)"/>
    <circle cx="60" cy="0" r="30" fill="url(#iconGrad)"/>
    <circle cx="0" cy="80" r="30" fill="url(#iconGrad)"/>

    <!-- Connection lines -->
    <line x1="0" y1="-50" x2="-42" y2="-21" stroke="url(#iconGrad)" stroke-width="8"/>
    <line x1="0" y1="-50" x2="42" y2="-21" stroke="url(#iconGrad)" stroke-width="8"/>
    <line x1="-42" y1="21" x2="0" y2="50" stroke="url(#iconGrad)" stroke-width="8"/>
    <line x1="42" y1="21" x2="0" y2="50" stroke="url(#iconGrad)" stroke-width="8"/>
    <line x1="-42" y1="-21" x2="-42" y2="21" stroke="url(#iconGrad)" stroke-width="8"/>
    <line x1="42" y1="-21" x2="42" y2="21" stroke="url(#iconGrad)" stroke-width="8"/>
  </g>
</svg>'''
    return svg

def main():
    import os

    # Create output directory if it doesn't exist
    output_dir = os.path.join(os.path.dirname(__file__), '..', 'src-tauri', 'icons')
    os.makedirs(output_dir, exist_ok=True)

    # Generate SVG
    svg_content = generate_icon_svg()
    svg_path = os.path.join(output_dir, 'icon.svg')

    with open(svg_path, 'w') as f:
        f.write(svg_content)

    print(f"✓ Generated SVG icon at: {svg_path}")
    print("\nNext steps:")
    print("1. Convert SVG to PNG using online tool or ImageMagick:")
    print("   - Online: https://cloudconvert.com/svg-to-png")
    print("   - Or install ImageMagick: brew install imagemagick")
    print("   - Then run: convert -background none -size 1024x1024 icon.svg icon.png")
    print("\n2. Generate Tauri icons:")
    print("   npm run tauri icon src-tauri/icons/icon.png")

if __name__ == '__main__':
    main()
