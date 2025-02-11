#!/bin/sh

# Set permissions for the data directory
chown -R clickhouse:clickhouse /var/lib/clickhouse

# Only process users.xml template since config.xml is now static
envsubst < /etc/clickhouse-server/users.xml.template > /etc/clickhouse-server/users.xml

# Start ClickHouse server
exec /entrypoint.sh

