<img src="https://app.listen-rs.com/listen-icon.png" width="250"/>

# Listen MCP Server 🔍

A Model Context Protocol (MCP) server that lets AI assistants like Claude use
the Listen API for crypto token data. This setup allows AI models to get
real-time Solana tokens information in a controlled and reliable way.

Example usage: https://claude.ai/share/b3a423ec-d953-4dce-9bc6-e2c6b1b10a77

## What is MCP? 🤔

The Model Context Protocol (MCP) is a system that lets AI apps, like Claude
Desktop, connect to external tools and data sources. It gives a clear and safe
way for AI assistants to work with local services and APIs while keeping the
user in control.

## What does this server do? 🚀

The Listen MCP server:

- Enables AI assistants to fetch real-time cryptocurrency price data
- Provides token metadata and market information
- Lets assistants retrieve price charts and historical data
- Delivers insights on top tokens with customizable filters

## Prerequisites 📋

Before you begin, ensure you have:

- Rust and Cargo installed (stable channel)
- Claude Desktop installed
- Git installed

Get Rust:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

You can verify your Rust installation by running:

```sh
rustc --version
cargo --version
```

## Installation 🛠️

Clone the repository:

```sh
git clone https://github.com/piotrostr/listen.git
cd listen/listen-mcp
```

Build the project:

```sh
cargo build --release
```

And make the binary available for execution:

On MacOS:

```sh
sudo cp ./target/release/listen-mcp /usr/local/bin
```

On Windows:

```sh
# Add the current release directory to your PATH permanently
$env:Path += ";$((Get-Location).Path)\target\release"
[Environment]::SetEnvironmentVariable("Path", $env:Path, [System.EnvironmentVariableTarget]::User)

# Or simply move the executable to a dedicated bin folder
# Create a bin directory if you don't have one
mkdir -Force C:\bin
copy .\target\release\listen-mcp.exe C:\bin\
# Then add C:\bin to your PATH if not already there
$env:Path += ";C:\bin"
[Environment]::SetEnvironmentVariable("Path", $env:Path, [System.EnvironmentVariableTarget]::User)
```

## Configuration ⚙️

Configure Claude Desktop to recognize the Listen MCP server
You can find `claude_desktop_config.json` inside the settings of Claude Desktop app:

- Open the Claude Desktop app and enable Developer Mode from the top-left menu bar.
- Once enabled, open Settings and navigate to the Developer Option, where you'll find the Edit Config button.

OR open claude_desktop_config.json from terminal:

For MacOS:

```sh
code ~/Library/Application\ Support/Claude/claude_desktop_config.json
```

For Windows:

```sh
code %APPDATA%\Claude\claude_desktop_config.json
```

Paste in:

```json
{
  "mcpServers": {
    "listen-mcp": {
      "command": "listen-mcp",
      "args": ["stdio"]
    }
  }
}
```

For the changes to take effect:

- Completely quit Claude Desktop (not just close the window)
- Start Claude Desktop again
- Look for the 🔌 icon to verify the Listen server is connected

## Usage 🎯

Once configured, you can ask Claude to perform crypto token operations. Here are
some example prompts:

- "Can you fetch the current price of SOL (So11111111111111111111111111111111111111112)?"
- "What are the top 5 tokens between 5m and 10m market cap in the last 2 hours?"
- "What is the Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump token?"

The server will:

- Process the token data request
- Query the Listen API with the appropriate parameters
- Return formatted results to Claude

## Features ✨

- Token Price Fetching: Get real-time prices for any token by mint address
- Top Tokens Analysis: Find trending tokens with customizable volume and market cap filters
- Price Charts: Retrieve historical price data with multiple interval options
- Token Metadata: Access on-chain and IPFS metadata for tokens

## Examples

- Independent research

<img src="https://github.com/user-attachments/assets/47e5c150-f9b5-4416-8d54-d28c1017a41e" alt="Sample Usage 1" style="max-width: 100%;">

- Price action analysis
  
<img src="https://github.com/user-attachments/assets/3e675be9-db4c-48a0-a24d-7a7afbdcf529" alt="Sample Usage 2" style="max-width: 100%;">
