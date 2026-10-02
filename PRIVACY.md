# Privacy Policy for AURORA Kaushal IDM

**Last updated:** October 2, 2026

AURORA Kaushal IDM ("we", "our", or "the extension") is an open-source, high-performance download accelerator extension developed for modern web browsers. We are committed to protecting your privacy. This Privacy Policy explains our practices regarding user data.

---

## 1. Zero Data Collection
AURORA Kaushal IDM does **not** collect, record, track, transmit, or sell any personal data, browsing history, or user information. 

Specifically, we do NOT collect:
- Personally identifiable information (name, email, address, phone number).
- Financial, banking, or payment data.
- Health, medical, or biometric information.
- Authentication credentials, passwords, or session tokens.
- Personal communications (emails, chat messages, messages).
- Geolocation or GPS coordinates.
- Browsing history, visited websites, or page metadata.
- User keystrokes, mouse tracking, or telemetry analytics.

---

## 2. Permissions and How They Are Used
AURORA Kaushal IDM requests only the minimum set of browser permissions strictly necessary for its core download acceleration functionality:

- **`downloads`**: Used to save completed files into your local Downloads directory and register tasks with the browser's download manager.
- **`storage`**: Used solely to persist your local user preferences (such as parallel connection count and auto-capture toggles) on your device via `chrome.storage.local`.
- **`contextMenus`**: Used to provide the right-click "Download with AURORA" shortcut menu on links, audio, video, and image files.
- **`alarms`**: Used to schedule background metric updates and keep download workers alive during active multi-stream transfers.
- **`notifications`**: Used to display local desktop notifications when a download completes or encounters an error.
- **`offscreen`**: Used in Manifest V3 to assemble multi-part binary file chunks in an isolated offscreen DOM environment without blocking browser performance.
- **`Host Permissions (<all_urls>)`**: Used solely to send parallel HTTP Range requests to the remote web servers hosting the files you explicitly choose to download.

---

## 3. Local Data Storage
All download state, configuration settings, and temporary binary chunks are stored strictly locally on your machine using standard browser APIs (`chrome.storage.local` and `IndexedDB`). No data is ever transmitted to any external server or third-party database.

---

## 4. No Remote Code Execution
All JavaScript files, CSS styling, and WebAssembly acceleration modules are packaged and bundled completely locally inside the extension. The extension does not load, fetch, or execute any remote scripts, external CDNs, or dynamic code.

---

## 5. Third-Party Sharing
We do not sell, rent, trade, or transfer any user data to third parties. We do not use third-party analytics (e.g. Google Analytics), advertising networks, or tracking SDKs.

---

## 6. Open Source Transparency
AURORA Kaushal IDM is open-source. The entire source code can be inspected and audited at:
[https://github.com/kaushalbro/aurora-kaushal-idm](https://github.com/kaushalbro/aurora-kaushal-idm)

---

## 7. Contact
If you have any questions or feedback regarding this Privacy Policy, please open an issue on our GitHub repository:
[https://github.com/kaushalbro/aurora-kaushal-idm/issues](https://github.com/kaushalbro/aurora-kaushal-idm/issues)
