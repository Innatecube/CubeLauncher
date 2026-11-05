# CubeLauncher Official Website

A modern, high-performance landing page for **CubeLauncher**, inspired by the dark gaming aesthetic and interactive presentation of **NoRisk Client** (`norisk.gg`).

## Key Features

- **Cyber Dark & Neon Cyan Aesthetic**: Matches CubeLauncher's native UI with glowing borders, glassmorphic panels, and sharp geometric styling.
- **Dynamic Stats Ribbon**: Highlights instant `< 0.2s` launch times, `~35MB` RAM usage, 3-in-1 account support, and mod loaders.
- **Interactive Feature Tabs**: Live tab switcher demonstrating the Mod Engine, 3D Wardrobe, Multi-Loader Instances, and Content Discovery with real screenshots.
- **Real-Time 3D Wardrobe Playground**: Interactive HTML5 Canvas widget where visitors can drag to rotate a 3D Minecraft character, tweak pitch/yaw, toggle classic vs. slim models, and control animation.
- **Comprehensive Benchmark Matrix**: Side-by-side comparison between CubeLauncher, the Official Launcher, Prism, and CurseForge.
- **Interactive FAQ Accordion**: Expandable Q&A section with clean accordion toggles.
- **One-Click Downloads**: Direct download links for both the Windows Installer (`CubeLauncher-Setup.exe`) and Portable executable (`CubeLauncher.exe`).

## Local Preview

### Option 1: Direct in Browser
Simply double-click or open `website/index.html` in your web browser (Chrome, Edge, Firefox, Brave).

### Option 2: Local HTTP Server
Run the following in PowerShell / terminal:
```powershell
python -m http.server -d website 8000
```
Then visit [http://localhost:8000](http://localhost:8000).

## Deployment

The website is 100% static (HTML, CSS, JS) and zero-dependency:
- **GitHub Pages**: Set the deployment source to `/website` or move the files to a `gh-pages` branch.
- **Vercel / Netlify / Cloudflare Pages**: Deploy the `website/` directory directly.
