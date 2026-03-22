#!/usr/bin/env bash
set -e

SERVICE_NAME="flexdns"
INSTALL_BIN_DIR="/usr/lib/flexdns"
INSTALL_BIN="$INSTALL_BIN_DIR/flexDNS"
SERVICE_DIR="/etc/systemd/system"
SERVICE_FILE="$SERVICE_DIR/flexDNS.service"
LOCAL_RESOLVER="127.0.0.10"
CONFIG_DIR="/etc/flexdns"
CONFIG_PATH="$CONFIG_DIR/resolv.conf"

echo "FlexDNS installer starting..."

if [ "$EUID" -ne 0 ]; then
    echo "Please run as root: sudo ./install.sh"
    exit 1
fi

echo "Installing binary to $INSTALL_BIN..."
mkdir -p "$INSTALL_BIN_DIR"
cp bin/flexDNS "$INSTALL_BIN"
chmod +x "$INSTALL_BIN"

mkdir -p "$CONFIG_DIR"

echo "Installing systemd service to $SERVICE_FILE..."
cp systemd/flexDNS.service "$SERVICE_FILE"

systemctl daemon-reload
systemctl enable "$SERVICE_NAME"
systemctl restart "$SERVICE_NAME"

echo "FlexDNS service installed and started."

echo "Setting system DNS to FlexDNS ($LOCAL_RESOLVER)..."

if [ -L /etc/resolv.conf ]; then
    RESOLV_TARGET=$(readlink -f /etc/resolv.conf)
    rm /etc/resolv.conf
    cat > /etc/resolv.conf <<EOF
nameserver $LOCAL_RESOLVER
options edns0
EOF
else
    cat > /etc/resolv.conf <<EOF
nameserver $LOCAL_RESOLVER
options edns0
EOF
fi

sudo mkdir -p /etc/NetworkManager/conf.d
sudo tee /etc/NetworkManager/conf.d/99-dns.conf > /dev/null <<EOF
[main]
dns=none
rc-manager=unmanaged
EOF

sudo systemctl restart NetworkManager
systemctl stop systemd-resolved || true
systemctl disable systemd-resolved || true

echo "DNS updated, resolv.conf link broken if it existed, NetworkManager/systemd-resolved disabled."

echo ""
echo "Installation complete."
echo "Binary path: $INSTALL_BIN"
echo "Config path used by FlexDNS: $CONFIG_PATH"
systemctl status "$SERVICE_NAME"