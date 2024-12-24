#!/bin/bash

# This script is designed to provision the xtee application on a Linux system.
# It performs the following tasks:
# 1. Ensures the script is run as root.
# 2. Defines color codes for logging.
# 3. Provides logging functions for different log levels (info, success, warning, error).
# 4. Sets up configuration variables such as USERNAME, PROJECT_REPO, PROJECT_OWNER, PROJECT_VERSION, INSTALL_DIR, BIN_DIR, and EXEC_FILE.
# 5. Defines the DOWNLOAD_URI for fetching the xtee application binary.
# 6. Checks if required commands are available on the system.
# 7. Downloads and installs the xtee application binary to the specified directory.
# 8. Ensures proper permissions and ownership for the installed files.
# 9. Provides error handling and logging throughout the script execution.

# Exit on error, undefined variables, and pipe failures
set -euo pipefail
trap 'echo "Error on line $LINENO. Exit code: $?"' ERR

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions with colors
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

log_success() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} ${GREEN}$1${NC}"
}

log_warning() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} ${YELLOW}WARNING: $1${NC}"
}

error() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} ${RED}ERROR: $1${NC}" >&2
    exit 1
}

# Configuration
USERNAME="xtee"
PROJECT_REPO="xtee"
PROJECT_OWNER="mossida"
PROJECT_VERSION="latest"

INSTALL_DIR="/home/${USERNAME}/.xtee"
BIN_DIR="${INSTALL_DIR}/bin"
EXEC_FILE="${BIN_DIR}/xtee"

DOWNLOAD_URI="https://github.com/${PROJECT_OWNER}/${PROJECT_REPO}/releases/${PROJECT_VERSION}/download/xtee"

# Check if script is run as root
if [[ $EUID -ne 0 ]]; then
    error "This script must be run as root"
fi

# Function to check command existence
check_command() {
    if ! command -v "$1" &> /dev/null; then
        error "Required command '$1' not found. Please install it first."
    fi
}

# Check for required commands
for cmd in curl adduser usermod apt; do
    check_command "$cmd"
    log_success "Found required command: $cmd"
done

log "Creating new user..."

# Create new user with adduser (more secure than using root's password)
if ! id "$USERNAME" &>/dev/null; then
    DEBIAN_FRONTEND=noninteractive adduser --gecos "" --disabled-password "$USERNAME" || 
        error "Failed to create user '$USERNAME'"
    
    # Generate a secure random password
    RANDOM_PASS=$(openssl rand -base64 12)
    echo "$USERNAME:$RANDOM_PASS" | chpasswd ||
        error "Failed to set password for user '$USERNAME'"
    
    log_success "Generated random password for '$USERNAME'. Please change it on first login."
else
    log_warning "User '$USERNAME' already exists"
fi

# Add user to required groups
for group in sudo video input audio dialout tty; do
    usermod -aG "$group" "$USERNAME" ||
        error "Failed to add user to group '$group'"
    log_success "Added user to group: $group"
done

log_success "User setup completed successfully"

# Update package list and install dependencies
log "Installing dependencies..."
apt-get update || error "Failed to update package list"

# Install dependencies with proper error handling
DEPS=(
    "libwebkit2gtk-4.1-0"
    "seatd"
    "labwc"
)

for dep in "${DEPS[@]}"; do
    log "Installing $dep..."
    apt-get install -y "$dep" ||
        error "Failed to install $dep"
    log_success "Successfully installed: $dep"
done

# Create installation directory if it doesn't exist
if [[ ! -d $BIN_DIR ]]; then
    mkdir -p "$BIN_DIR" ||
        error "Failed to create install directory '$BIN_DIR'"
    log_success "Created installation directory: $BIN_DIR"
fi

# Download and verify the binary
log "Downloading XTEE..."
curl --fail --location --progress-bar --output "$EXEC_FILE" "$DOWNLOAD_URI" ||
    error "Failed to download XTEE from '$DOWNLOAD_URI'"
log_success "Download completed successfully"

# Set proper ownership and permissions
chown -R "$USERNAME:$USERNAME" "$INSTALL_DIR" ||
    error "Failed to set ownership of '$INSTALL_DIR'"
log_success "Set correct ownership for installation directory"

chmod 755 "$EXEC_FILE" ||
    error "Failed to set permissions on '$EXEC_FILE'"
log_success "Set correct permissions for executable"

# Verify the binary is executable
if ! [[ -x "$EXEC_FILE" ]]; then
    error "Failed to make '$EXEC_FILE' executable"
fi

log_success "Installation completed successfully"
log_success "XTEE has been installed to: $EXEC_FILE"




