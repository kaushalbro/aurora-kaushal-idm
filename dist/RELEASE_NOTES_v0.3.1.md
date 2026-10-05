# AURORA Kaushal IDM v0.3.1 — Release Notes

## 🚀 What's New in v0.3.1

### 1. 🔍 Dynamic Response & Content-Disposition Downloads
- Full support for URLs without static file extensions (`.php`, `.aspx`, `.jsp`, or query params).
- RFC 5987 / RFC 6266 `filename*=UTF-8''...` & standard `filename="..."` parsing.
- Real-time MIME type to file extension inference (`application/pdf` -> `.pdf`, `application/x-iso9660-image` -> `.iso`).
- Session credentials (`credentials: 'include'`) forwarding for authenticated dynamic invoice & document downloads.

### 2. 🌐 Chrome Web Store & Browser Permissions Compliance
- Updated MV3 extension manifests to v0.3.1.
- Documented privacy practices and permission justifications (`activeTab`, `downloads`, `storage`, `contextMenus`, `alarms`, `notifications`, `offscreen`, `host_permissions`).

### 3. 🖥️ Desktop App
- React 19 + Tailwind CSS + Tauri 2.0.
- Cross-platform autostart, system tray integration, and universal URL protocol interception.

---

### 📦 Checksums & Assets
Verify files with `dist/SHA256SUMS.txt`.
