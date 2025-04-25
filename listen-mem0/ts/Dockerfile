FROM oven/bun:1.0.30

WORKDIR /app

# Copy package files
COPY package.json bun.lockb ./

# Install dependencies
RUN bun install

# Copy application files
COPY . .

# Expose the port the app runs on
EXPOSE 9696

# Command to run the application
CMD ["bun", "run", "index.ts"] 