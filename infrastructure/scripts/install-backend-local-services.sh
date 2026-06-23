#!/bin/bash
set -euo pipefail

fail() {
  echo "ERROR: $*" >&2
  exit 1
}

if [[ "$EUID" -ne 0 ]]; then
  fail "Run with sudo: sudo ./infrastructure/scripts/install-backend-local-services.sh"
fi

TARGET_USER="${SUDO_USER:-}"
if [[ -z "$TARGET_USER" || "$TARGET_USER" == "root" ]]; then
  fail "Run this via sudo from the macOS user that owns the repo."
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TARGET_HOME="$(dscl . -read "/Users/$TARGET_USER" NFSHomeDirectory | awk '{print $2}')"

BACKEND_BIN="$REPO_ROOT/apps/backend/target/release/epsx"
RUN_SCRIPT="$REPO_ROOT/infrastructure/scripts/run-backend-instance.sh"
TUNNEL_TEMPLATE="$REPO_ROOT/infrastructure/cloudflare/cloudflared-config.backend-local.yml"
ENV_DIR="$REPO_ROOT/.secret/backend"
LOG_DIR="$TARGET_HOME/Library/Logs/epsx"
LAUNCHD_DIR="/Library/LaunchDaemons"
CF_ETC_DIR="/etc/cloudflared"
CF_TUNNEL_ID="6bee9b58-eede-4b4c-815c-94c0ee38fe58"
CF_USER_CRED="$TARGET_HOME/.cloudflared/$CF_TUNNEL_ID.json"
CF_SYSTEM_CRED="$CF_ETC_DIR/$CF_TUNNEL_ID.json"

command -v cloudflared >/dev/null 2>&1 || fail "cloudflared is not installed. Install with: brew install cloudflared"
[[ -x "$BACKEND_BIN" ]] || fail "Backend binary missing. Run ./infrastructure/scripts/build-backend-local.sh first."
[[ -x "$RUN_SCRIPT" ]] || fail "Run script missing or not executable: $RUN_SCRIPT"
[[ -f "$TUNNEL_TEMPLATE" ]] || fail "Tunnel template missing: $TUNNEL_TEMPLATE"

for env_name in dev staging prod; do
  [[ -f "$ENV_DIR/$env_name.env" ]] || fail "Missing env file: $ENV_DIR/$env_name.env"
done

mkdir -p "$LOG_DIR" "$CF_ETC_DIR"
chown "$TARGET_USER":staff "$LOG_DIR"

if [[ -f "$CF_USER_CRED" && ! -f "$CF_SYSTEM_CRED" ]]; then
  cp "$CF_USER_CRED" "$CF_SYSTEM_CRED"
fi
[[ -f "$CF_SYSTEM_CRED" ]] || fail "Missing tunnel credentials file: $CF_SYSTEM_CRED"

cp "$TUNNEL_TEMPLATE" "$CF_ETC_DIR/config.yml"
chmod 600 "$CF_SYSTEM_CRED"
chmod 644 "$CF_ETC_DIR/config.yml"

cloudflared tunnel ingress validate --config "$CF_ETC_DIR/config.yml"

write_backend_plist() {
  local label="$1"
  local env_name="$2"
  local plist_path="$LAUNCHD_DIR/$label.plist"

  cat > "$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$label</string>
    <key>UserName</key>
    <string>$TARGET_USER</string>
    <key>WorkingDirectory</key>
    <string>$REPO_ROOT</string>
    <key>ProgramArguments</key>
    <array>
        <string>$RUN_SCRIPT</string>
        <string>$env_name</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>ProcessType</key>
    <string>Background</string>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/backend-$env_name.out.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/backend-$env_name.err.log</string>
</dict>
</plist>
EOF

  chmod 644 "$plist_path"
  plutil -lint "$plist_path" >/dev/null
}

write_port_bridge_plist() {
  local label="com.epsx.port-bridge"
  local plist_path="$LAUNCHD_DIR/$label.plist"

  cat > "$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$label</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/bash</string>
        <string>-lc</string>
        <string>/opt/homebrew/bin/socat TCP-LISTEN:8080,fork,reuseaddr TCP:127.0.0.1:18080 &amp; /opt/homebrew/bin/socat TCP-LISTEN:4810,fork,reuseaddr TCP:127.0.0.1:28080 &amp; /opt/homebrew/bin/socat TCP-LISTEN:9180,fork,reuseaddr TCP:127.0.0.1:38080 &amp; wait</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>ProcessType</key>
    <string>Background</string>
    <key>StandardOutPath</key>
    <string>/tmp/port-bridge.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/port-bridge.err</string>
</dict>
</plist>
EOF

  chmod 644 "$plist_path"
  plutil -lint "$plist_path" >/dev/null
}

# wave49(slice-1): pay.epsx.io port-bridge. Routes
#   4747 (host) → 30082 (colima NodePort) → epsx-pay-svc:8103
#   4748 (host) → 30083 (colima NodePort) → epsx-pay-bff:3002
# The Cloudflare Tunnel ingress for pay.epsx.io points at
# http://localhost:4747.
write_pay_port_bridge_plist() {
  local label="com.epsx.pay-port-bridge"
  local plist_path="$LAUNCHD_DIR/$label.plist"

  cat > "$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$label</string>
    <key>ProgramArguments</key>
    <array>
        <string>/bin/bash</string>
        <string>-lc</string>
        <string>/opt/homebrew/bin/socat TCP-LISTEN:4747,fork,reuseaddr TCP:127.0.0.1:30082 &amp; /opt/homebrew/bin/socat TCP-LISTEN:4748,fork,reuseaddr TCP:127.0.0.1:30083 &amp; wait</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>ProcessType</key>
    <string>Background</string>
    <key>StandardOutPath</key>
    <string>/tmp/pay-port-bridge.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/pay-port-bridge.err</string>
</dict>
</plist>
EOF

  chmod 644 "$plist_path"
  plutil -lint "$plist_path" >/dev/null
}

for entry in "com.epsx.backend.dev dev" "com.epsx.backend.staging staging" "com.epsx.backend.prod prod"; do
  label="${entry%% *}"
  env_name="${entry##* }"
  write_backend_plist "$label" "$env_name"

  if launchctl print "system/$label" >/dev/null 2>&1; then
    launchctl bootout "system/$label" || true
  fi

  launchctl bootstrap system "$LAUNCHD_DIR/$label.plist"
  launchctl enable "system/$label"
  launchctl kickstart -k "system/$label"
done

write_port_bridge_plist

if launchctl print system/com.epsx.port-bridge >/dev/null 2>&1; then
  launchctl bootout system/com.epsx.port-bridge || true
fi

launchctl bootstrap system "$LAUNCHD_DIR/com.epsx.port-bridge.plist"
launchctl enable system/com.epsx.port-bridge
launchctl kickstart -k system/com.epsx.port-bridge

# wave49(slice-1): pay.epsx.io port-bridge.
write_pay_port_bridge_plist

if launchctl print system/com.epsx.pay-port-bridge >/dev/null 2>&1; then
  launchctl bootout system/com.epsx.pay-port-bridge || true
fi

launchctl bootstrap system "$LAUNCHD_DIR/com.epsx.pay-port-bridge.plist"
launchctl enable system/com.epsx.pay-port-bridge
launchctl kickstart -k system/com.epsx.pay-port-bridge

if [[ ! -f "$LAUNCHD_DIR/com.cloudflare.cloudflared.plist" ]]; then
  cloudflared service install
fi

if launchctl print system/com.cloudflare.cloudflared >/dev/null 2>&1; then
  launchctl kickstart -k system/com.cloudflare.cloudflared
else
  launchctl bootstrap system "$LAUNCHD_DIR/com.cloudflare.cloudflared.plist"
  launchctl enable system/com.cloudflare.cloudflared
  launchctl kickstart -k system/com.cloudflare.cloudflared
fi

cat <<EOF
Installed local backend boot services.

LaunchDaemons:
  com.epsx.backend.dev      -> 127.0.0.1:18080 -> dev-api.epsx.io
  com.epsx.backend.staging  -> 127.0.0.1:28080 -> staging-api.epsx.io
  com.epsx.backend.prod     -> 127.0.0.1:38080 -> api.epsx.io
  com.epsx.port-bridge      -> 8080/4810/9180 -> 18080/28080/38080
  com.epsx.pay-port-bridge  -> 4747/4748 -> 30082/30083 -> pay.epsx.io
  com.cloudflare.cloudflared -> /etc/cloudflared/config.yml

Logs:
  $LOG_DIR/backend-dev.out.log
  $LOG_DIR/backend-staging.out.log
  $LOG_DIR/backend-prod.out.log
  /tmp/pay-port-bridge.log
  /Library/Logs/com.cloudflare.cloudflared.out.log

Health checks:
  curl http://127.0.0.1:18080/health
  curl http://127.0.0.1:28080/health
  curl http://127.0.0.1:38080/health
  curl http://127.0.0.1:30082/health      # pay-svc
  curl http://127.0.0.1:30083/api/health  # pay-bff
EOF
