#!/bin/bash

# Exit on error
set -e

# Check if git is installed
if ! command -v git &> /dev/null; then
    echo "Git is not installed. Installing git..."
    sudo apt-get update
    sudo apt-get install -y git
fi

# Clone repository
echo "Cloning repository..."
git clone https://github.com/yourusername/yourrepository.git

# Change directory permissions
sudo chown -R pi:pi yourrepository

echo "Repository cloned successfully!"
