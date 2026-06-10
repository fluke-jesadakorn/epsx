#!/bin/sh
set -e

md5pw() {
  printf "md5%s" "$(printf '%s%s' "$2" "$1" | md5sum | cut -d' ' -f1)"
}

cat > /etc/pgbouncer/userlist.txt <<EOF
"${DB_USER}" "$(md5pw "${DB_USER}" "${DB_PASSWORD}")"
"pgbouncer_admin" "$(md5pw "pgbouncer_admin" "${PGBOUNCER_ADMIN_PASSWORD}")"
EOF

exec pgbouncer /etc/pgbouncer/pgbouncer.ini
