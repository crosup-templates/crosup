#!/bin/bash

# Define the release information
RELEASE_URL="https://api.github.com/repos/tsirysndr/crosup/releases/latest"
ASSET_NAME="_x86_64-unknown-linux-gnu.tar.gz"

# Retrieve the download URL for the desired asset
DOWNLOAD_URL=$(curl -sSL $RELEASE_URL | grep -o "browser_download_url.*$ASSET_NAME\"" | cut -d ' ' -f 2)

ASSET_NAME=$(basename $DOWNLOAD_URL)

# Define the installation directory
INSTALL_DIR="/usr/local/bin"

DOWNLOAD_URL=`echo $DOWNLOAD_URL | tr -d '\"'`

# Download the asset
wget $DOWNLOAD_URL -O /tmp/$ASSET_NAME

# Extract the asset
tar -xzf /tmp/$ASSET_NAME -C /tmp

# Move the extracted binary to the installation directory
sudo mv /tmp/crosup $INSTALL_DIR

# Set the correct permissions for the binary
chmod +x $INSTALL_DIR/crosup

# Clean up temporary files
rm /tmp/$ASSET_NAME

echo "Installation completed! 🎉"

crosup install