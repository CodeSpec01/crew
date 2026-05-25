# Crew 🚢 (Tauri Desktop Edition)

> **Status:** Active Development (WIP) - Currently engineering Phase 3: Synchronized Local Video Streaming (HTTP Range Requests + WebSocket Sync).

Crew is an offline-first, decentralized peer-to-peer (P2P) workspace. It utilizes WebRTC to establish direct mesh networks between local devices, completely bypassing external internet requirements for data transmission. This Tauri wrapper brings the React-based Crew web application natively to the desktop with minimal overhead, integrating seamlessly with the mobile Termux Switchboard.

## Core Features

* **The Global Lobby:** Real-time presence tracking of all devices on the local network.
* **Secure Mesh Rooms:** Isolated 2D routing matrix (`peersRef.current[roomId][peerId]`) allowing users to dynamically spin up multi-user communication channels.
* **Direct P2P Messaging:** Instant, zero-latency text communication via WebRTC data channels.
* **Uncapped File Sharing:** Share massive binary files directly between hard drives. Features a custom chunking algorithm (16KB limits) with a WebRTC backpressure monitor (`peer._channel.bufferedAmount`) to prevent RAM crashes and network choking.
* **Self-Healing Topology:** Automatic late-joiner dialing and graceful UI cleanup upon abrupt peer disconnections via Socket.io heartbeats.

## Architecture

Crew operates on a hybrid topology:

1. **The Switchboard (Light Lane):** A centralized Node.js/Socket.io server (typically hosted via Termux on an Android device) acts purely as a signaling server. It passes WebRTC offers/answers and announces room joins/leaves. It **does not** touch private data.
2. **The Mesh (Heavy Lane):** Once handshakes are complete, devices form a decentralized WebRTC mesh. All text and file transfers route directly from device to device.

## Tech Stack

* **Frontend Engine:** React.js, Tailwind CSS (or standard CSS)
* **Desktop Wrapper:** Tauri (Rust)
* **P2P Networking:** `simple-peer` (WebRTC)
* **Signaling:** `socket.io-client`
* **Polyfills:** `buffer`, `process` (Required for Node.js APIs within the browser environment)

## Prerequisites

Before building the Tauri application, ensure your development environment is fully configured:

1. **Rust & Cargo:** Required for Tauri. Install via [rustup](https://rustup.rs/).
2. **Node.js & npm/yarn:** For the React frontend.
3. **OS-Specific Dependencies:** * *Windows:* Visual Studio C++ Build Tools.
* *Linux:* `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`.
* *macOS:* Xcode Command Line Tools.



## Installation & Setup

1. **Clone the repository:**
```bash
git clone <repository-url>
cd crew-tauri

```


2. **Install frontend dependencies:**
```bash
npm install

```


3. **Configure the Switchboard IP:**
Open your main App/Socket configuration file (e.g., `src/App.js`) and ensure the `SERVER_URL` points to your active Termux Node.js server IP:
```javascript
const SERVER_URL = 'http://<YOUR_LOCAL_IP>:3000'; 

```


4. **Run in Development Mode:**
This command starts the React development server and opens the native Tauri window.
```bash
npm run tauri dev

```


5. **Build for Production:**
To compile the final `.exe`, `.app`, or `.deb` standalone binary:
```bash
npm run tauri build

```



## Development Notes

* **Polyfill Requirement:** Because `simple-peer` was originally designed for Node environments, the React frontend requires global polyfills for `Buffer` and `process`. Ensure these are declared at the top of your React entry point.
* **Network Isolation:** For WebRTC to successfully punch through local NAT, ensure the machine running the Tauri app and the machine running the Switchboard are on the same local subnet, and that your OS firewall allows traffic on port 3000.

---

**Author:** Aviral Gaur