#!/bin/bash
# Init additional databases for Staging
# Mounted as /docker-entrypoint-initdb.d/init-databases.sh in postgres container

set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE DATABASE epsx_analytics_staging;
    CREATE DATABASE epsx_notifications_staging;
    CREATE DATABASE epsx_payments_staging;
EOSQL

echo "Staging databases created: epsx_analytics_staging, epsx_notifications_staging, epsx_payments_staging"
