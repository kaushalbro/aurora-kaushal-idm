# Chrome Web Store Listing: AURORA Kaushal IDM

## 1. Store Metadata

* **Extension Name**: AURORA Kaushal IDM
* **Short Description** (max 132 chars):
  Accelerate file downloads with adaptive multi-connection streams, dynamic chunk splitting, and live visual progress tracking.
* **Category**: Productivity / Developer Tools
* **Language**: English
* **Version**: 0.3.1

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
* 🛡️ **Zero Native Installation Required**: Runs 100% inside your browser with pure WebAssembly.

---

## 3. Permissions Justifications (Privacy Practices Tab)

Enter the following justifications in the **Privacy practices** tab of the Chrome Web Store Developer Dashboard:

| Permission | Exact Justification (Copy & Paste) |
| :--- | :--- |
| `activeTab` | `Required to detect downloadable media, document links, and capture the current page URL when the user explicitly clicks the extension popup or initiates a download from the active tab.` |
| `downloads` | `Required to save completed multi-stream accelerated files directly into the user's download directory and register them in the browser download history.` |
| `storage` | `Required to persist user preferences locally, including connection concurrency, dynamic segment size, and auto-capture file filters.` |
| `contextMenus` | `Required to provide the right-click "Download with AURORA" context menu option on links, images, video streams, and audio elements.` |
| `alarms` | `Required to schedule periodic background metric synchronization, download speed calculations, and connection health checks.` |
| `notifications` | `Required to notify the user when a large file download has successfully finished or if an integrity checksum mismatch is detected.` |
| `offscreen` | `Required to run WebAssembly (WASM) multithreaded chunking and binary stream assembly within an isolated background offscreen document in Manifest V3.` |
| `host_permissions` (`<all_urls>`) | `Required to send parallel HTTP Range requests to the remote file servers hosting the files the user chooses to download.` |

---

## 4. Privacy & Data Use Disclosure

* **Single Purpose Description**: High-performance multi-stream download acceleration and management using WebAssembly.
* **Data Collection**: No personal data, browsing history, or user data is collected, logged, or transferred to external servers.
* **Network Activity**: All network requests are made directly between your browser and the remote server hosting the file you chose to download.
* **Storage**: Preferences and temporary download state are stored strictly locally on your device via `chrome.storage.local`.

---

## 5. Build and Packaging

To build the latest release package ready for Chrome Web Store upload:
```bash
# Run the automated build script:
bash scripts/build_extension.sh

# The ready-to-upload zip is created at:
# dist/aurora-chrome-v0.3.1.zip (or dist/aurora-extension-v0.3.1.zip)
```
