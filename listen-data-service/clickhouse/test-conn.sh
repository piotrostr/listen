#!/bin/bash

# echo 'SELECT 1' | curl "https://listen-clickhouse.fly.dev:8443/?user=default&password=S96Gz6U0BkYS&database=default" \
#     -H "Content-Type: application/json" \
#     --data-binary @-

clickhouse-client \
    --user default \
    --password "$CLICKHOUSE_PASSWORD" \
    --database default \
    --host listen-clickhouse.fly.dev \
    --port 8443 \
    --secure

