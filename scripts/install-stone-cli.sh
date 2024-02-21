#!/usr/bin/env bash

set -eo pipefail

VERSION="v0.1.0-rc2"
INSTALL_DIR="${HOME}/.stone/${VERSION}"
TARBALL="stone-cli-linux-x86-64.tar.gz"

while true; do
  case "$1" in
    -i | --install-dir ) INSTALL_DIR="$2"; shift 2 ;;
    * ) break ;;
  esac
done

echo "Installing Stone in ${INSTALL_DIR}..."
mkdir -p "${INSTALL_DIR}"
wget https://github.com/Moonsong-Labs/stone-prover-cli/releases/download/${VERSION}/${TARBALL} -O "${INSTALL_DIR}/${TARBALL}"
tar -xf "${INSTALL_DIR}/${TARBALL}"

echo "Configuring permissions..."
chmod +x "${INSTALL_DIR}/cpu_air_prover"
chmod +x "${INSTALL_DIR}/cpu_air_verifier"
chmod +x "${INSTALL_DIR}/stone-prover-cli"

# Add the tool to the PATH
echo "Configuring PATH..."
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
  PROFILE_FILE=""
  # ZSH_NAME is set on zsh
  if [ -v ZSH_NAME ]; then
    PROFILE_FILE="${HOME}/.zsh"
  elif [ -v BASH ]; then
    PROFILE_FILE="${HOME}/.bashrc"
  else
    echo "Unsupported shell, you will need to add the export PATH statement in the right configuration file manually."
  fi

  if [ -n "${PROFILE_FILE}" ]; then
    echo -e "\n# Stone prover and verifier\nexport PATH=\"${INSTALL_DIR}:\$PATH\"" >> "${PROFILE_FILE}"
  fi
fi

# Notify the user to update the PATH immediately
echo "Done!"
echo "Stone was added to ${PROFILE_FILE} and will be available the next time you open a shell."
echo "To add Stone to your PATH immediately, run the following command:"
echo "export PATH=\"${INSTALL_DIR}:\$PATH\""