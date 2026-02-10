#!/bin/bash
# Init additional databases for Development
# Mounted as /docker-entrypoint-initdb.d/init-databases.sh in postgres container

set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE DATABASE epsx_analytics_dev;
    CREATE DATABASE epsx_notifications_dev;
    CREATE DATABASE epsx_payments_dev;
EOSQL

echo "✅ Development databases created: epsx_analytics_dev, epsx_notifications_dev, epsx_payments_dev"
