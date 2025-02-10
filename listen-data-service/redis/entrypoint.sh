#!/bin/sh
# Replace environment variables in redis.conf
envsubst < /usr/local/etc/redis/redis.conf.template > /usr/local/etc/redis/redis.conf
# Start Redis with the processed config
exec redis-server /usr/local/etc/redis/redis.conf
