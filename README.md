# socks5-bridge

A local HTTP-to-SOCKS5 proxy bridge for Chrome on macOS.

**Problem:** Chrome doesn't support SOCKS5 username/password authentication, so you can't point it at `socks5://user:pass@host:port` and expect it to work.

**Solution:** `socks5-bridge` runs a local HTTP proxy on loopback. Chrome connects to it using HTTP proxy config it already understands, and the bridge handles the authenticated SOCKS5 upstream on Chrome's behalf.

## Quick start

```bash
# Build
cargo build --release

# Create a config file (edit credentials + upstream host)
cp config.sample.toml config.toml

# Validate config
cargo run -- check --config config.toml

# Run in foreground
cargo run -- start --config config.toml --foreground

# Run as daemon (background)
cargo run -- start --config config.toml
```

## CLI

| Command | Description |
|---------|-------------|
| `start --config <FILE>` | Start the bridge (daemonizes by default; use `--foreground` to stay in terminal) |
| `start --config <FILE> --validate-only` | Load and validate config, then exit |
| `check --config <FILE>` | Validate config without opening any network listeners |
| `test-upstream --config <FILE>` | Test the upstream SOCKS5 connection and auth path |
| `status [--admin-port]` | Query the admin API for runtime state |
| `stop [--admin-port]` | Gracefully shut down a running daemon via admin API |
| `print-chrome-args --config <FILE>` | Print `--proxy-server=...` flags to pass to Chrome |

## Configuration

All options live in a single TOML file. Copy `config.sample.toml` and edit:

```toml
[listener]
host = "127.0.0.1"
port = 8899

[upstream]
type = "socks5"
host = "your-upstream.example.com"
port = 1080
username = "your-username"
password = "your-password"
connect_timeout_ms = 8000
auth_timeout_ms = 5000
command_timeout_ms = 8000
remote_dns = true

[policy]
allow_loopback_only = true
allow_private_destinations = false
allow_localhost_destinations = false
max_concurrent_connections = 256
idle_timeout_ms = 60000
graceful_shutdown_timeout_ms = 5000

[logging]
level = "info"
format = "json"
# file = "/path/to/logfile.log"
redact_credentials = true

[health]
enable_admin_api = true
admin_host = "127.0.0.1"
admin_port = 8898
probe_host = "example.com"
probe_port = 443
probe_interval_ms = 30000
```

### Key settings

- **`remote_dns`** — when `true` (default), hostnames are sent to the upstream SOCKS5 server for resolution rather than resolved locally. This keeps DNS consistent with the upstream's view of the network.
- **`allow_loopback_only`** — when `true`, the listener binds only to loopback. Set to `false` with extreme caution.
- **`redact_credentials`** — strips username/password from log output.

## Architecture

```
Chrome (HTTP proxy) → listener (loopback TCP) → HTTP proxy parser
  → policy check → SOCKS5 client (auth + connect) → upstream SOCKS5
  → relay (bidirectional copy)
```

| Module | Role |
|--------|------|
| `listener` | Binds TCP, accept loop, spawns per-connection handler |
| `http_proxy` | Parses HTTP proxy requests (GET + CONNECT), policy check, SOCKS5 connect, relay |
| `socks5` | SOCKS5 protocol types and handshake/connect client |
| `relay` | `tokio::io::copy_bidirectional` between client and upstream |
| `policy` | Destination allow/deny (loopback-only, private-range blocking) |
| `session` | Per-connection state machine and counters |
| `admin_api` | Axum HTTP server for health/metrics on admin port |
| `config` | TOML config loading and validation |
| `logging` | Tracing subscriber init (JSON or plain, optional file, credential redaction) |

## Chrome setup

Print the flags you need:

```bash
socks5-bridge print-chrome-args --config config.toml
```

Then launch Chrome with those flags, or configure Chrome's proxy settings to point at `127.0.0.1:8899` as an HTTP proxy.

## Running as a LaunchAgent

A sample `com.local.socks5-bridge.plist` is included. Copy it to `~/Library/LaunchAgents/`, update the paths to point at your binary and config, then:

```bash
launchctl load ~/Library/LaunchAgents/com.local.socks5-bridge.plist
```

## Admin API

When `health.enable_admin_api = true`, the following endpoints are available on the admin port (default `8898`):

| Endpoint | Description |
|----------|-------------|
| `GET /health` | Returns `{"status": "ok"}` or `{"status": "degraded"}` |
| `GET /metrics` | JSON snapshot of runtime counters |
| `POST /shutdown` | Graceful shutdown |

## Requirements

- Rust edition 2024 (stable)
- macOS (loopback binding; daemonization uses `fork`)
