# ⚠️ MIRROR REPOSITORY WARNING

> [!WARNING]
> **This is a read-only mirror repository automatically synced from a Forgejo instance.**
> 
> - **Issues, Pull Requests, and Discussions should NOT be created here**
> - **This repository is for reference and distribution purposes only**
> - **All development happens on the primary Forgejo repository**
> 
> For contributions, bug reports, or questions, please contact the repository owner through other channels.

---

# Simple Web Editor (SWE) – Two-Part Architecture

## 1. Editor GUI  
**E.g. URL:** `https://swe.example.com` or `http://127.0.0.1:8080`
**Tech:** Rust **Yew** (compiled to WebAssembly)

- Presents a **file-browser sidebar** and an in-browser **code editor**.  
- Lets you create, rename, move, delete and edit any file that belongs to the site.  
- Saves changes by calling the SWE API over HTTPS (JSON / WebSocket).

## 2. Site Renderer & API  
**E.g. URL:** `https://example.com` or `http://127.0.0.1:8000`
**Tech:** Rust **Rocket**

- Exposes all API routes used by the GUI.  
- Renders the public site at the root domain.  
- Serves static assets from `/public_site` (e.g., images, CSS, JS).

## Features

- 🎨 **Customizable Syntax Highlighting** - Create and manage custom themes with live preview
- 🔄 **Git Integration** - Built-in Git operations with automatic synchronization
- 💾 **Persistent Theme Storage** - Themes saved to server with localStorage fallback
- 🔐 **Authentication** - Secure token-based authentication system
- 📁 **File Management** - Complete file browser with create, edit, move, delete operations

# Testing
Build the Dockerfile
```bash
docker run --rm -it -p 80:80/tcp -p 8000:8000/tcp -p 8080:8080/tcp -e ROCKET_ADDRESS=0.0.0.0 -e ROCKET_PORT=8000 -e API_URL=http://127.0.0.1:8000/ -e EDITOR_URL=http://127.0.0.1:8080/ -e ADMIN_TOKEN=secret123 -e ROCKET_LOG_LEVEL=debug simpleweb:latest
```

API will be served at `http://127.0.0.1:8000/` but EDITOR will be at `http://127.0.0.1:80/`

---

**🔗 This repository is automatically mirrored from a private Forgejo instance.**