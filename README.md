<!-- PROJECT LOGO -->
<br />
<div align="center">
 <a href="https://git.prisma.moe/aichan/simple_web_editor">
 </a>

 <h1 align="center">Simple Web Editor</h1>
 <p align="center"> <img 
    src="https://visitcounter.aichan.ovh/counter/simple_web_editor/svg?label=Project%20Visits" height=20
    alt="Visit Counter" /> </p>

 <p align="center">
 A two-part web-based code editor built with Rust. Features a Yew WebAssembly frontend and Rocket backend API for real-time file editing and site rendering.
 <br />
 <br />
 <a href="https://yew.rs/docs/getting-started/introduction">Yew Documentation</a>
 ·
 <a href="https://rocket.rs/guide/v0.5/">Rocket Documentation</a>
 ·
 <a href="https://git.prisma.moe/aichan/simple_web_editor/issues">Report Bug</a>
 ·
 <a href="https://git.prisma.moe/aichan/simple_web_editor/issues">Request Feature</a>
 </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
 <summary>Table of Contents</summary>
 <ol>
 <li><a href="#about-the-project">About The Project</a></li>
   <ul>
      <li><a href="#built-with">Built With</a></li>
   </ul>
 </li>
 <li>
 <a href="#architecture">Architecture</a>
 <ul>
 <li><a href="#editor-gui">Editor GUI</a></li>
 <li><a href="#site-renderer--api">Site Renderer & API</a></li>
 </ul>
 </li>
 <li><a href="#features">Features</a></li>
 <li><a href="#deployment">Deployment</a></li>
   <ul>
      <li><a href="#using-docker">Using Docker</a></li>
   </ul>
   <ul>
      <li><a href="#using-docker-compose">Using Docker Compose</a></li>
   </ul>
 <li><a href="#contributing">Contributing</a></li>
 <li><a href="#license">License</a></li>
 <li><a href="#contact">Contact</a></li>
 </ol>
</details>

## About The Project

Simple Web Editor (SWE) is a lightweight, web-based code editor designed for static site development: a self-hosted alternative to GitHub Pages. It uses a two-part architecture optimized for real-time web editing, built with Rust technologies and containerized for easy deployment.

The editor provides an intuitive interface for managing files and editing code with syntax highlighting. Repositories can be pulled directly from the interface, making version control and collaboration effortless.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Built With
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white) ![WebAssembly](https://img.shields.io/badge/WebAssembly-654FF0?style=for-the-badge&logo=webassembly&logoColor=white) [![Yew](https://img.shields.io/badge/Yew-2E8B57?style=for-the-badge&logo=rust&logoColor=white)](#) [![Rocket](https://img.shields.io/badge/Rocket-D22128?style=for-the-badge&logo=rocket&logoColor=white)](#) ![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white) [![Visual Studio Code](https://custom-icon-badges.demolab.com/badge/Visual%20Studio%20Code-0078d7.svg?style=for-the-badge&logo=vsc&logoColor=white)](#)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Architecture

Simple Web Editor uses a two-part architecture that separates the editor interface from the site rendering and API services.

### Editor GUI
**Access:** `https://swe.example.com` or `http://127.0.0.1:80`  
**Technology:** Rust Yew (compiled to WebAssembly)

- Presents a **file-browser sidebar** and an in-browser **code editor**
- Allows you to create, rename, move, delete and edit any file that belongs to the site
- Features syntax highlighting and customizable themes
- Saves changes by calling the SWE API over HTTPS (JSON / WebSocket)

### Site Renderer & API
**Access:** `https://example.com` or `http://127.0.0.1:8000`  
**Technology:** Rust Rocket

- Exposes all API routes used by the GUI
- Renders the public site at the root domain
- Serves static assets from `/public_site` (e.g., images, CSS, JS)
- Handles authentication and file operations

<p align="right">(<a href="#architecture">back to top</a>)</p>

## Features

- 🎨 **Customizable Syntax Highlighting** - Create and manage custom themes with live preview
- 🔄 **Git Integration** - Built-in Git operations with automatic synchronization
- 💾 **Persistent Theme Storage** - Themes saved to server with localStorage fallback
- 🔐 **Authentication** - Secure token-based authentication system
- 📁 **File Management** - Complete file browser with create, edit, move, delete operations
- ⚡ **Real-time Updates** - WebSocket-based live updates and collaboration
- 🌐 **WebAssembly Performance** - Fast, native-speed execution in the browser

<p align="right">(<a href="#features">back to top</a>)</p>

## Deployment

You have multiple options to deploy Simple Web Editor:

### Using Docker

Run the pre-built container:
```bash
docker run --rm -it \
  -p 80:80/tcp \
  -p 8000:8000/tcp \
  -p 8080:8080/tcp \
  -e ROCKET_ADDRESS=0.0.0.0 \
  -e ROCKET_PORT=8000 \
  -e API_URL=http://127.0.0.1:8000/ \
  -e EDITOR_URL=http://127.0.0.1:8080/ \
  -e ADMIN_PASSWORD=<YOUR_PASSWORD> \
  -e ROCKET_LOG_LEVEL=debug \
  git.prisma.moe/aichan/simple_web_editor:latest
```

### Using Docker Compose

Create a `docker-compose.yml` file:
```yaml
services:
  simple_web:
    container_name: simple_web_editor
    image: git.prisma.moe/aichan/simple_web_editor:latest
    ports:
      - <YOUR_PORT>:8000 # API and Renderer
      - <YOUR_PORT>:80   # Editor
    environment:
      - ROCKET_ADDRESS=0.0.0.0
      - ROCKET_PORT=8000
      - ADMIN_PASSWORD=<YOUR_PASSWORD>
      - API_URL=<YOUR_API_URL>
      - EDITOR_URL=<YOUR_EDITOR_URL>
    volumes:
      - ./simple_web_editor/public_site:/public_site
    restart: unless-stopped
```

Then run:
```bash
docker-compose up -d
```

**Access Points:**
- API will be served at `http://127.0.0.1:8000/`
- Editor will be served at `http://127.0.0.1:80/`

<p align="right">(<a href="#deployment">back to top</a>)</p>

## Contributing

Contributions are welcome! Please fork the repository, make your changes, and open a pull request.

1. Fork the Project on [Forgejo](https://git.prisma.moe/aichan/simple_web_editor)
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#contributing">back to top</a>)</p>

## License

Distributed under the European Union Public License v1.2. See `LICENSE` for more information.

<p align="right">(<a href="#license">back to top</a>)</p>

## Contact

Aitor Astorga Saez de Vicuña - a.astorga.sdv@protonmail.com

Project Link: [https://git.prisma.moe/aichan/simple_web_editor](https://git.prisma.moe/aichan/simple_web_editor)

<p align="right">(<a href="#contact">back to top</a>)</p>

## Acknowledgments

Thanks to these amazing projects and technologies!

- [Rust Yew](https://yew.rs/) - A modern Rust framework for creating multi-threaded front-end web apps with WebAssembly
- [Rocket](https://rocket.rs/) - A web framework for Rust that makes it simple to write fast, secure web applications
- [WebAssembly](https://webassembly.org/) - A binary instruction format for a stack-based virtual machine

<p align="right">(<a href="#readme-top">back to top</a>)</p>
