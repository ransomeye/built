#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/ransomeye_intelligence/install_dependencies.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Installs Python dependencies required for Phase 3 artifact generation

set -e

echo "Installing Python dependencies for Phase 3 artifact generation..."

# Check if pip is available
if ! command -v pip3 &> /dev/null; then
    echo "pip3 not found. Attempting to install..."
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y python3-pip
    elif command -v yum &> /dev/null; then
        sudo yum install -y python3-pip
    else
        echo "ERROR: Cannot install pip. Please install pip3 manually."
        exit 1
    fi
fi

# Install required packages
echo "Installing required Python packages..."
pip3 install --user scikit-learn numpy shap sentence-transformers faiss-cpu cryptography

echo "✓ Dependencies installed successfully"

