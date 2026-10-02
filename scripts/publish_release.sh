#!/usr/bin/env bash
# ==============================================================================
# AURORA Kaushal IDM - GitHub Release Automation Script
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
VERSION="v0.1.0"

echo "============================================================"
echo " 🚀 AURORA Kaushal IDM - GitHub Release Publisher"
echo "    Target Release: ${VERSION}"
echo "============================================================"

# Ensure all packages are built
if [[ ! -f "${DIST_DIR}/aurora-kaushal-idm_0.1.0_amd64.deb" ]] || [[ ! -f "${DIST_DIR}/aurora-kaushal-idm-v0.1.0-windows-x64.zip" ]]; then
  echo "Building all packages first..."
  bash "${SCRIPT_DIR}/build_extension.sh" all
  bash "${SCRIPT_DIR}/package_desktop.sh"
fi

echo -e "\nAvailable release assets in dist/:"
ls -lh "${DIST_DIR}"/aurora-*

echo -e "\n============================================================"
echo " Publishing Options:"
echo " 1. GitHub Actions (Automated CI/CD):"
echo "    git tag -a ${VERSION} -m 'Release ${VERSION}'"
echo "    git push origin ${VERSION}"
echo "    -> GitHub Actions will automatically build & attach all binaries!"
echo ""
echo " 2. GitHub CLI (gh):"
echo "    gh release create ${VERSION} dist/* --title 'AURORA Kaushal IDM ${VERSION}' --notes-file dist/RELEASE_NOTES_v0.1.0.md"
echo ""
echo " 3. GitHub Web UI:"
echo "    Go to your GitHub repository -> Releases -> 'Draft a new release'"
echo "    Tag: ${VERSION}"
echo "    Drag and drop all files from ${DIST_DIR}/ into the assets area!"
echo "============================================================"
