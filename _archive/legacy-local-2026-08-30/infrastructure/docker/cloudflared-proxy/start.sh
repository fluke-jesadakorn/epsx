#!/bin/sh
set -e

echo "Setting up iptables transparent proxy rules..."
iptables -t nat -N REDSOCKS 2>/dev/null || true

# Skip local/private networks
iptables -t nat -A REDSOCKS -d 0.0.0.0/8 -j RETURN
iptables -t nat -A REDSOCKS -d 10.0.0.0/8 -j RETURN
iptables -t nat -A REDSOCKS -d 127.0.0.0/8 -j RETURN
iptables -t nat -A REDSOCKS -d 169.254.0.0/16 -j RETURN
iptables -t nat -A REDSOCKS -d 172.16.0.0/12 -j RETURN
iptables -t nat -A REDSOCKS -d 192.168.0.0/16 -j RETURN
iptables -t nat -A REDSOCKS -p tcp -j REDIRECT --to-ports 12345
iptables -t nat -A OUTPUT -p tcp -j REDSOCKS

echo "iptables rules applied. Starting redsocks..."
redsocks -c /etc/redsocks.conf &
sleep 1

echo "Starting cloudflared (will retry until CF edge is reachable)..."
exec cloudflared tunnel --config /etc/cloudflared/config.yml run
