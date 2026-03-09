#!/bin/bash
# Init additional databases for Development
# Mounted as /docker-entrypoint-initdb.d/init-databases.sh in postgres container

set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- Ensure epsx_user role exists
    DO \$\$
    BEGIN
      IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'epsx_user') THEN
        CREATE ROLE epsx_user LOGIN CREATEDB PASSWORD '${EPSX_DB_PASSWORD:-password}';
      ELSE
        ALTER ROLE epsx_user CREATEDB;
      END IF;
    END \$\$;

    -- Create databases owned by epsx_user
    CREATE DATABASE epsx_analytics_dev OWNER epsx_user;
    CREATE DATABASE epsx_notifications_dev OWNER epsx_user;
    CREATE DATABASE epsx_payments_dev OWNER epsx_user;

    -- Ensure main DB is also owned by epsx_user
    ALTER DATABASE epsx_dev OWNER TO epsx_user;
EOSQL

# Set default privileges in each database
for db in epsx_dev epsx_analytics_dev epsx_notifications_dev epsx_payments_dev; do
  psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$db" <<-EOSQL
    ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO epsx_user;
    ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO epsx_user;
    ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON FUNCTIONS TO epsx_user;
EOSQL
done

echo "✅ Development databases created (owned by epsx_user): epsx_analytics_dev, epsx_notifications_dev, epsx_payments_dev"
