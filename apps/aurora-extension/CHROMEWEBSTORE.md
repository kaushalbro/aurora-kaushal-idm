# Chrome Web Store Listing: AURORA Kaushal IDM

## 1. Store Metadata

* **Extension Name**: AURORA Kaushal IDM
* **Short Description** (max 132 chars):
  Accelerate file downloads with adaptive multi-connection streams, dynamic chunk splitting, and live visual progress tracking.
* **Category**: Productivity / Developer Tools
* **Language**: English
* **Version**: 0.1.0

---

## 2. Detailed Store Description

AURORA Kaushal IDM is a high-performance, in-browser download manager engineered with WebAssembly. It accelerates downloads by splitting files into multiple parallel byte streams, dynamically balancing connection speeds, and displaying live download segments in real time.

### Key Features
* ⚡ **Multi-Stream Download Acceleration**: Splits supporting downloads across 4 to 32 parallel connections for faster downloads.
* 🧠 **Adaptive Stream Scheduling**: Dynamically balances connections and reallocates straggler segments to ensure maximum bandwidth saturation.
* 📊 **Live Segment Visualizer**: Visualizes chunk progress with real-time colored byte blocks and speed indicators directly in the browser popup.
* 🖱️ **One-Click Context Menu**: Right-click on any downloadable link, image, audio, or video and select "Download with AURORA".
* ⚙️ **Automatic Download Capture**: Intercepts large archive, media, and binary files and replaces slow single-stream browser downloads.
* 🔒 **Built-in Hash Verification**: Verifies SHA-256 and BLAKE3 file checksums before saving to guarantee file integrity.
* 🛡️ **Zero Native Installation**: Runs 100% inside your browser with WebAssembly — no desktop software, background daemons, or extra setup required.

---

## 3. Permissions Justifications

| Permission | Justification |
| :--- | :--- |
| `downloads` | Required to register completed downloads into the browser's download manager and trigger file saving to the user's download directory. |
| `storage` | Required to save user preferences, such as default connection count, auto-capture toggles, and file extension filters. |
| `contextMenus` | Required to provide the right-click "Download with AURORA" context menu option on links, images, and media. |
| `alarms` | Required to maintain background service worker timers and periodically refresh active download metrics. |
| `notifications` | Required to notify the user when a large file download has finished or if an integrity checksum mismatch occurs. |
| `host_permissions` (`<all_urls>`) | Required to send parallel HTTP Range requests to the remote file servers hosting the files the user chooses to download. |

---

## 4. Privacy & Data Use Disclosure

* **Data Collection**: None. AURORA does not collect, log, or transmit any browsing history, personal data, or analytics to external servers.
* **Network Activity**: All network requests are made directly between your browser and the remote server hosting the file you chose to download.
* **Storage**: Preferences and temporary download state are stored strictly locally on your device via `chrome.storage.local`.

---

## 5. Build and Packaging

To build the latest release package ready for Chrome Web Store upload:
```bash
# Run the automated build script:
./scripts/build_extension.sh

# The ready-to-upload zip will be created at:
# dist/aurora-extension-v0.1.0.zip
```
