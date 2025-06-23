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

