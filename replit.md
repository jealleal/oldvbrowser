# Roblox Browser

A web browser running inside Roblox using Rust and headless Chrome.

## Project Overview

This project consists of:
- **Rust HTTP Server**: Runs a headless Chrome browser and serves frames via HTTP
- **Roblox Client**: Uses long-polling to fetch browser frames and render them as an image in Roblox

## Architecture

- **Backend**: Rust server using `headless_chrome` crate
  - Listens on port 3000
  - Starts a headless browser instance
  - Streams screen captures using a custom binary protocol
  - Communicates with Roblox clients via HTTP

- **Roblox Side**: Luau scripts in `roblox/` directory
  - Polls the HTTP server for frame updates
  - Renders frames as EditableImage
  - Sends input events (mouse, keyboard) to the server

## Current Setup

- Language: Rust (2021 edition)
- Build system: Cargo
- Deployment: Configured for VM deployment
- Port: 3000 (console output)

## To Publish (Make Public)

Click the **Publish** button in Replit to make this server publicly accessible with a URL.

## Dependencies

- Rust dependencies: See Cargo.toml (headless_chrome, image, rayon, etc.)
- System: Chromium (installed in Replit environment)
- Roblox tools: Managed by Aftman (rojo, selene, lune)
