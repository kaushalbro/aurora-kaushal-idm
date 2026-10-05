# Aurora Kaushal Download Manager - Nepal (v3.0.3)

## 🚀 What's New in v3.0.3

### ⚡ Critical Fixes & Enhancements
- **Complete `mime-db` Integration (2,500+ MIME Types)**:
  - Integrated `mime-db` containing all standard IANA and media MIME types.
  - Added full support for JPEG (`image/jpeg`, `image/jpg`, `image/pjpeg`), PNG, WebP, GIF, SVG, BMP, AVIF, HEIC, TIFF, audio, video, documents, and archives.
  - Automatically translates backend script endpoints (`get_avoir_pdf.php`, `image.php?id=...`) into their actual media file extensions based on response Content-Type headers.
- **Resolved Duplicate Range Over-Fetching**:
  - Eliminated the bug where streams downloaded duplicate bytes when faster workers completed early.
  - Enforced strict non-overlapping partition locks in both the Rust WebAssembly core and JavaScript fallback engine.
  - Downloads now complete with 100% byte-exact accuracy (`totalBytes === downloadedBytes`).
- **Eliminated Self-Interception**:
  - Added guards (`downloadItem.byExtensionId === chrome.runtime.id`) to prevent AURORA from intercepting or canceling its own saved files.
- **Session Credentials & Cookies**:
  - Added `credentials: 'include'` to all `fetch()` calls (`probe()`, `fetchRange()`, `downloadSingleStream()`) so protected and session-authenticated downloads (cloud storage, invoices, user portals) succeed without authentication errors.
- **Dynamic Link & Button Capture**:
  - Enhanced content script capture for dynamic download endpoints (e.g. `get_avoir_pdf.php`, `download.php?id=...`, `data-download-url`).

---

## 📦 Release Assets & Checksums

| Asset | Platform / Target | SHA-256 Checksum |
| :--- | :--- | :--- |
| `aurora-chrome-v3.0.3.zip` | Google Chrome Extension | `23dce257bdd58afce1dc7acfa18f211b73fd433e36e37209e94e09bc065db7c2` |
| `aurora-brave-v3.0.3.zip` | Brave Browser Extension | `23dce257bdd58afce1dc7acfa18f211b73fd433e36e37209e94e09bc065db7c2` |
| `aurora-edge-v3.0.3.zip` | Microsoft Edge Extension | `23dce257bdd58afce1dc7acfa18f211b73fd433e36e37209e94e09bc065db7c2` |
| `aurora-firefox-v3.0.3.xpi` | Mozilla Firefox Addon | `2c8da20a028cde1a98bb26ee4ed473c1ea1e57bc31bfa72d6e5330e24a3a5b8b` |
| `aurora-safari-v3.0.3.zip` | Apple Safari Extension | `799ac1f48dbe736bd585bb94cef82e0f9c33ec3bc1c31a45504ecbd10b00fcac` |
| `aurora-kaushal-idm_3.0.3_amd64.deb` | Linux (Debian / Ubuntu / Mint) | `f049cfce314642c3ec66bb8ce100d2523a3b897ea61e05f7063722f50f9502f7` |
| `aurora-kaushal-idm-v3.0.3-setup.exe` | Windows Installer (NSIS) | `184588a9d88a4533ef0f6e0f0357febcaef63c35c849839d6ea78751a72e5d52` |
| `aurora-kaushal-idm-v3.0.3-windows-x64.zip` | Windows Standalone Portable | `97ef93b959d379379a112d9151166a21291b45920e6dbf6dee2e7d902813152e` |
| `aurora-kaushal-idm-v3.0.3-linux-x86_64.tar.gz` | Linux Portable Binary | `1bccc90a8633d263bf245d25bbd1d0fb0332457ff792ba64ceb371041aa981c5` |
| `aurora-kaushal-idm-v3.0.3-macos.zip` | macOS App Bundle | `2ff8da8e474bf51fec69342120c3af9e2edfabce7f5e09d9dbdafab3ba4f6a2c` |
