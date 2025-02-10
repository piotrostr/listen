FROM clickhouse/clickhouse-server:latest

# Install envsubst
RUN apt-get update && apt-get install -y gettext-base && rm -rf /var/lib/apt/lists/*

# Copy config files as templates
COPY config.xml /etc/clickhouse-server/config.xml.template
COPY users.xml /etc/clickhouse-server/users.xml.template

# Copy and make the entrypoint script executable
COPY entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/entrypoint.sh

# Ensure the clickhouse user owns the data directory
RUN mkdir -p /var/lib/clickhouse && chown -R clickhouse:clickhouse /var/lib/clickhouse

# Use the entrypoint script
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
