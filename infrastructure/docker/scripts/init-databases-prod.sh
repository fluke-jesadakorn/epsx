#!/bin/bash
# Init additional databases for Production
# Mounted as /docker-entrypoint-initdb.d/init-databases.sh in postgres container

set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE DATABASE epsx_analytics_prod;
    CREATE DATABASE epsx_notifications_prod;
    CREATE DATABASE epsx_payments_prod;
EOSQL

echo "✅ Production databases created: epsx_analytics_prod, epsx_notifications_prod, epsx_payments_prod"
