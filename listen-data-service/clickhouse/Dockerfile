FROM clickhouse/clickhouse-server:latest

# Install envsubst
RUN apt-get update && apt-get install -y gettext-base && rm -rf /var/lib/apt/lists/*

# Create necessary directories
RUN mkdir -p /var/log/clickhouse-server \
	/var/lib/clickhouse/tmp \
	/var/lib/clickhouse/format_schemas \
	/var/lib/clickhouse/user_files

# Set proper permissions
RUN chown -R clickhouse:clickhouse \
	/var/log/clickhouse-server \
	/var/lib/clickhouse

# Copy config files directly (not as templates)
COPY config.xml /etc/clickhouse-server/config.xml
COPY users.xml /etc/clickhouse-server/users.xml.template

# Copy and make the entrypoint script executable
COPY entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/entrypoint.sh

# Use the entrypoint script
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
