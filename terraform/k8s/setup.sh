#!/bin/bash

# Function to encode string to base64
encode_base64() {
    echo -n "$1" | base64
}

set +e
source listen-data-service/.env
set -e

# Read and encode secrets from terraform.tfvars
RPC_URL=$(encode_base64 $RPC_URL)
WS_URL=$(encode_base64 $WS_URL)
GEYSER_URL=$(encode_base64 $GEYSER_URL)
GEYSER_X_TOKEN=$(encode_base64 $GEYSER_X_TOKEN)

# Update the secrets in 03-indexer.yaml
sed -i.bak \
    -e "s|\${BASE64_ENCODED_RPC_URL}|${RPC_URL}|g" \
    -e "s|\${BASE64_ENCODED_WS_URL}|${WS_URL}|g" \
    -e "s|\${BASE64_ENCODED_GEYSER_URL}|${GEYSER_URL}|g" \
    -e "s|\${BASE64_ENCODED_GEYSER_X_TOKEN}|${GEYSER_X_TOKEN}|g" \
    03-indexer.yaml

# Copy config files from listen-data-service
echo "Copying Redis config..."
REDIS_CONF=$(cat ../listen-data-service/redis/redis.conf)
sed -i.bak "s|# Contents from ../listen-data-service/redis/redis.conf will go here|$REDIS_CONF|" 01-redis.yaml

echo "Copying Clickhouse configs..."
CLICKHOUSE_CONFIG=$(cat ../listen-data-service/clickhouse/config.xml)
CLICKHOUSE_USERS=$(cat ../listen-data-service/clickhouse/users.xml)
sed -i.bak \
    -e "s|# Contents from ../listen-data-service/clickhouse/config.xml will go here|$CLICKHOUSE_CONFIG|" \
    -e "s|# Contents from ../listen-data-service/clickhouse/users.xml will go here|$CLICKHOUSE_USERS|" \
    02-clickhouse.yaml

# Clean up backup files
rm *.bak

# Apply the Kubernetes configurations in order
echo "Applying Kubernetes configurations..."
kubectl apply -f 00-namespace.yaml
kubectl apply -f 01-redis.yaml
kubectl apply -f 02-clickhouse.yaml
sleep 10  # Wait for Redis and Clickhouse to start
kubectl apply -f 03-indexer.yaml
kubectl apply -f 04-adapter.yaml

echo "Deployment complete!" 