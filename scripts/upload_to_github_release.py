#!/usr/bin/env python3
"""
AURORA Kaushal IDM - Direct GitHub Release Asset Uploader
Automatically creates or updates a GitHub Release and uploads all distribution
packages (.deb, .exe, .zip, .tar.gz, .xpi) from the dist/ folder.
"""

import os
import sys
import json
import glob
import urllib.request
import urllib.error

REPO_OWNER = "kaushalbro"
REPO_NAME = "aurora-kaushal-idm"
TAG_NAME = "v0.2.0"
RELEASE_TITLE = "AURORA Kaushal IDM v0.2.0 - Official Multi-Platform Release"

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ROOT_DIR = os.path.dirname(SCRIPT_DIR)
DIST_DIR = os.path.join(ROOT_DIR, "dist")
RELEASE_NOTES_PATH = os.path.join(DIST_DIR, "RELEASE_NOTES_v0.2.0.md")


def get_token():
    token = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")
    if not token:
        print("=" * 60)
        print(" 🔑 GitHub Authentication Required")
        print("=" * 60)
        print(" To upload binaries to GitHub Releases, please enter a GitHub Personal")
        print(" Access Token (PAT) with 'repo' / 'contents:write' permissions.")
        print(" (Create one in 10s at: https://github.com/settings/tokens/new?scopes=repo)")
        print("=" * 60)
        try:
            token = input(" Enter your GitHub Token: ").strip()
        except EOFError:
            token = ""
    if not token:
        print("\n❌ Error: No GitHub Token provided. Set GITHUB_TOKEN environment variable or enter token.")
        sys.exit(1)
    return token


def make_request(url, method="GET", data=None, headers=None):
    req = urllib.request.Request(url, method=method)
    if headers:
        for k, v in headers.items():
            req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, data=data) as response:
            res_body = response.read()
            return response.status, res_body
    except urllib.error.HTTPError as e:
        err_body = e.read()
        return e.code, err_body
    except Exception as e:
        return 500, str(e).encode()


def get_or_create_release(token):
    auth_headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "User-Agent": "AURORA-Release-Uploader/1.0"
    }

    # 1. Check if release for tag already exists
    url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/tags/{TAG_NAME}"
    status, body = make_request(url, headers=auth_headers)

    if status == 200:
        release_data = json.loads(body.decode("utf-8"))
        print(f"✅ Found existing release for tag '{TAG_NAME}' (ID: {release_data['id']})")
        return release_data

    # 2. Read release notes
    notes = ""
    if os.path.exists(RELEASE_NOTES_PATH):
        with open(RELEASE_NOTES_PATH, "r", encoding="utf-8") as f:
            notes = f.read()
    else:
        notes = f"Official Multi-Platform release of AURORA Kaushal IDM {TAG_NAME}."

    # 3. Create new release
    print(f"Creating new GitHub Release '{RELEASE_TITLE}' for tag '{TAG_NAME}'...")
    create_url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases"
    payload = json.dumps({
        "tag_name": TAG_NAME,
        "target_commitish": "master",
        "name": RELEASE_TITLE,
        "body": notes,
        "draft": False,
        "prerelease": False
    }).encode("utf-8")

    auth_headers["Content-Type"] = "application/json"
    status, body = make_request(create_url, method="POST", data=payload, headers=auth_headers)

    if status in (200, 201):
        release_data = json.loads(body.decode("utf-8"))
        print(f"🎉 Created GitHub Release successfully (ID: {release_data['id']})!")
        return release_data
    else:
        print(f"❌ Failed to create release (HTTP {status}):\n{body.decode('utf-8')}")
        sys.exit(1)


def upload_asset(token, release_id, file_path):
    filename = os.path.basename(file_path)
    filesize = os.path.getsize(file_path)
    filesize_mb = filesize / (1024 * 1024)

    auth_headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "User-Agent": "AURORA-Release-Uploader/1.0"
    }

    # Check if asset already exists in release
    get_url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/{release_id}/assets"
    status, body = make_request(get_url, headers=auth_headers)
    if status == 200:
        assets = json.loads(body.decode("utf-8"))
        for asset in assets:
            if asset.get("name") == filename:
                print(f"  Removing previous asset '{filename}' (ID: {asset['id']})...")
                del_url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/assets/{asset['id']}"
                make_request(del_url, method="DELETE", headers=auth_headers)

    print(f"  ⬆️ Uploading {filename} ({filesize_mb:.2f} MB)...", end="", flush=True)

    upload_url = f"https://uploads.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/{release_id}/assets?name={urllib.parse.quote(filename)}"
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/octet-stream",
        "Content-Length": str(filesize),
        "User-Agent": "AURORA-Release-Uploader/1.0"
    }

    with open(file_path, "rb") as f:
        file_bytes = f.read()

    status, body = make_request(upload_url, method="POST", data=file_bytes, headers=headers)

    if status in (200, 201):
        print(" ✅ Done!")
        return True
    else:
        print(f" ❌ Failed (HTTP {status}): {body.decode('utf-8')}")
        return False


def main():
    print("============================================================")
    print(" 🚀 AURORA Kaushal IDM - GitHub Release Asset Publisher")
    print(f"    Repository: {REPO_OWNER}/{REPO_NAME}")
    print(f"    Release Tag: {TAG_NAME}")
    print("============================================================")

    if not os.path.isdir(DIST_DIR):
        print(f"❌ Error: dist directory '{DIST_DIR}' not found. Run build first.")
        sys.exit(1)

    token = get_token()
    release_data = get_or_create_release(token)
    release_id = release_data["id"]

    # Collect desktop packages only (excluding browser extensions)
    patterns = [
        "aurora-kaushal-idm*.deb",
        "aurora-kaushal-idm*windows*.zip",
        "aurora-kaushal-idm*macos*.zip",
        "aurora-kaushal-idm*.tar.gz",
        "SHA256SUMS.txt"
    ]

    files_to_upload = []
    for pattern in patterns:
        files_to_upload.extend(glob.glob(os.path.join(DIST_DIR, pattern)))

    # Ensure no extension files are included
    files_to_upload = [
        f for f in sorted(list(set(files_to_upload)))
        if not any(ext in os.path.basename(f) for ext in ["chrome", "brave", "edge", "firefox", "safari", ".xpi"])
    ]

    if not files_to_upload:
        print("❌ No desktop application files found in dist/ to upload.")
        sys.exit(1)

    print(f"\nFound {len(files_to_upload)} release files in dist/:\n")
    success_count = 0
    for f in files_to_upload:
        if upload_asset(token, release_id, f):
            success_count += 1

    print("\n" + "=" * 60)
    print(f" 🎉 SUCCESS! {success_count}/{len(files_to_upload)} assets published to GitHub Release!")
    print(f" 🔗 View your live release at:")
    print(f"    https://github.com/{REPO_OWNER}/{REPO_NAME}/releases/tag/{TAG_NAME}")
    print("=" * 60)


if __name__ == "__main__":
    main()
