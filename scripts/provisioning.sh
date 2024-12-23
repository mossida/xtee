#!/bin/bash

# Exit on error
set -e

apt install -y dbus-user-session
apt install -y libpam-systemd
apt install -y seatd
apt install -y xdg-user-dirs
apt install -y wayfire
apt install -y libgl1-mesa-dri
apt install -y libwebkit2gtk-4.1-0

echo "Provisioning..."
