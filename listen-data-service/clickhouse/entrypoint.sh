#!/bin/sh

# Set permissions for the data directory
chown -R clickhouse:clickhouse /var/lib/clickhouse

# Replace environment variables in config files
envsubst < /etc/clickhouse-server/config.xml.template > /etc/clickhouse-server/config.xml
envsubst < /etc/clickhouse-server/users.xml.template > /etc/clickhouse-server/users.xml

# Start ClickHouse server
exec /entrypoint.sh

