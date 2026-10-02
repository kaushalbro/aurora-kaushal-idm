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

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ROOT_DIR = os.path.dirname(SCRIPT_DIR)
DIST_DIR = os.path.join(ROOT_DIR, "dist")

def load_env_file():
    env_path = os.path.join(ROOT_DIR, ".env")
    if os.path.exists(env_path):
        with open(env_path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    k, v = line.split("=", 1)
                    k = k.strip()
                    v = v.strip().strip("\"'")
                    if k not in os.environ:
                        os.environ[k] = v

load_env_file()

VERSION_RAW = os.environ.get("AURORA_VERSION") or os.environ.get("VERSION") or "0.3.0"
VERSION = VERSION_RAW if VERSION_RAW.startswith("v") else f"v{VERSION_RAW}"

REPO_OWNER = os.environ.get("REPO_OWNER", "kaushalbro")
REPO_NAME = os.environ.get("REPO_NAME", "aurora-kaushal-idm")
TAG_NAME = VERSION
RELEASE_TITLE = f"AURORA Kaushal IDM {VERSION} - Official Modern Multi-Platform Release"
RELEASE_NOTES_PATH = os.path.join(DIST_DIR, f"RELEASE_NOTES_{VERSION}.md")


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


def delete_old_releases(token):
    auth_headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "User-Agent": "AURORA-Release-Uploader/1.0"
    }
    url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases"
    status, body = make_request(url, headers=auth_headers)
    if status == 200:
        releases = json.loads(body.decode("utf-8"))
        for rel in releases:
            if rel.get("tag_name") != TAG_NAME:
                print(f"🗑️ Deleting old release '{rel.get('tag_name')}' (ID: {rel['id']})...")
                del_url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/{rel['id']}"
                make_request(del_url, method="DELETE", headers=auth_headers)


def get_or_create_release(token):
    auth_headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "User-Agent": "AURORA-Release-Uploader/1.0"
    }

    # 1. Check if release for tag already exists
    url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/tags/{TAG_NAME}"
    status, body = make_request(url, headers=auth_headers)

    # 2. Read release notes
    notes = ""
    if os.path.exists(RELEASE_NOTES_PATH):
        with open(RELEASE_NOTES_PATH, "r", encoding="utf-8") as f:
            notes = f.read()
    else:
        notes = f"Official Multi-Platform release of AURORA Kaushal IDM {TAG_NAME}."

    if status == 200:
        release_data = json.loads(body.decode("utf-8"))
        print(f"✅ Found existing release for tag '{TAG_NAME}' (ID: {release_data['id']})")
        # Update release notes
        update_url = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/{release_data['id']}"
        payload = json.dumps({
            "name": RELEASE_TITLE,
            "body": notes,
        }).encode("utf-8")
        auth_headers["Content-Type"] = "application/json"
        make_request(update_url, method="PATCH", data=payload, headers=auth_headers)
        return release_data

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
    delete_old_releases(token)
    release_data = get_or_create_release(token)
    release_id = release_data["id"]

    # Collect all release files from dist
    files_to_upload = [
        os.path.join(DIST_DIR, f) for f in os.listdir(DIST_DIR)
        if os.path.isfile(os.path.join(DIST_DIR, f)) and not f.startswith("RELEASE_NOTES")
    ]

    files_to_upload.sort()

    if not files_to_upload:
        print("❌ No distribution files found in dist/ to upload.")
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
