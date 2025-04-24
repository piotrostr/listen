# listen-0mem

To install dependencies:

```bash
bun install
```

To run:

```bash
bun run index.ts
```

## Installing as a Service

1. Make sure you have the necessary environment variables in a `.env` file
2. Run the installation script:

```bash
chmod +x install-service.sh
sudo ./install-service.sh
```

3. Check service status:

```bash
sudo systemctl status listen-0mem
```

4. View logs:

```bash
sudo journalctl -u listen-0mem -f
```

This project was created using `bun init` in bun v1.1.30. [Bun](https://bun.sh) is a fast all-in-one JavaScript runtime.
