# Time API - High-Performance Rust Implementation

A high-performance time API service built in Rust with GPS PPS integration support, optimized for Raspberry Pi deployments.

**Original Python implementation:** https://github.com/tfoote000/http-time-api

## Features

- ⚡ **10x faster than Python/FastAPI** - <1ms p50 latency, >10k req/s throughput
- 💾 **Minimal memory footprint** - <20MB RSS under load
- 🎯 **Preserves exact API contract** from the original Python version
- 🕐 **Multiple timezone support** - Convert time to any IANA timezone
- 📊 **Time quality metrics** - Optional chrony integration for stratum, offset, etc.
- 💓 **Health checks** - `/health` and `/ready` endpoints for monitoring
- 📡 **MQTT publishing** (optional) - PPS pulses and health status
- 🚀 **Fast startup** - <100ms cold start
- 🛡️ **Security hardened** - HSTS, CSP, frame protection, etc.
- 🔄 **Graceful shutdown** - SIGTERM/SIGINT handling

## Architecture

### Production Deployment (Recommended)

```
Internet → Reverse Proxy (nginx/Caddy) → Time API (HTTP)
          ↓ (TLS termination)
          ↓ (HTTP/2, compression, caching)
```

The Time API runs on HTTP by default. Use a reverse proxy like nginx or Caddy for:
- TLS termination (HTTPS)
- HTTP/2 and HTTP/3 support
- Gzip/Brotli compression
- Rate limiting
- Caching
- Load balancing

### Direct TLS (Optional)

For testing HTTP/2 or HTTP/3 features directly (not recommended for production):

```
Internet → Time API (HTTPS with built-in TLS)
```

Requires setting `TLS_CERT_PATH` and `TLS_KEY_PATH` environment variables.

## API Endpoints

### `GET /` - Documentation

Returns HTML documentation page describing all endpoints.

### `GET /times` - Get Current Time

Get current time in one or more timezones.

**Query Parameters:**
- `tz` (optional): Comma-separated list of IANA timezone names. Default: `UTC`
- `include_quality` (optional): Include chrony time quality metrics. Default: `false`

**Example:**

```bash
curl "http://localhost:8463/times?tz=UTC,America/Denver,Asia/Tokyo"
```

**Response:**

```json
{
  "unix": 1234567890,
  "zones": {
    "UTC": {
      "local": "2009-02-13T23:31:30",
      "offset": 0
    },
    "America/Denver": {
      "local": "2009-02-13T16:31:30",
      "offset": -25200
    }
  }
}
```

**With quality metrics:**

```bash
curl "http://localhost:8463/times?tz=UTC&include_quality=true"
```

```json
{
  "unix": 1234567890,
  "zones": {
    "UTC": {
      "local": "2009-02-13T23:31:30",
      "offset": 0
    }
  },
  "time_quality": {
    "stratum": 1,
    "offset_seconds": 0.000000012,
    "reference_id": "PPS",
    "leap_status": "Normal"
  }
}
```

### `GET /health` - Health Check

Check system health and time synchronization status.

**Response:**

```json
{
  "status": "healthy",
  "checks": {
    "system_clock": {"status": "ok"},
    "chrony": {"status": "ok"}
  },
  "time_quality": {
    "stratum": 1,
    "offset_seconds": 0.000000012,
    "reference_id": "PPS",
    "leap_status": "Normal"
  }
}
```

**Status values:**
- `healthy`: All checks passed, stratum < 4
- `degraded`: Checks passed but stratum 4-15, or chrony unavailable
- `unhealthy`: Check failed or stratum 16 (unsynced)

### `GET /ready` - Readiness Check

Simple liveness check for Kubernetes/monitoring. Returns HTTP 200 if server is running.

## Configuration

All configuration is via environment variables. See `deployment/systemd/time-api.env` for examples.

### HTTP Server

- `PORT` (default: `8463`) - HTTP server port
- `HOST` (default: `0.0.0.0`) - Bind address
- `LOG_LEVEL` (default: `info`) - Logging level (error, warn, info, debug, trace)

### TLS (Optional - For HTTP/2 and HTTP/3)

**Note:** Not needed for production. Use a reverse proxy instead.

- `TLS_CERT_PATH` - Path to TLS certificate file (PEM format)
- `TLS_KEY_PATH` - Path to TLS private key file (PEM format)

### MQTT (Optional)

- `MQTT_BROKER` - MQTT broker URL (e.g., `mqtt://localhost:1883`)
- `MQTT_USERNAME` (optional) - MQTT authentication username
- `MQTT_PASSWORD` (optional) - MQTT authentication password
- `MQTT_BASE_TOPIC` (default: `time-api`) - Base topic for all publishes

**MQTT Topics:**
- `<base_topic>/pps` - Unix timestamp published every second
- `<base_topic>/health` - Health status published on change (max every 5s)

## Build

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- For ARM cross-compilation: `sudo apt-get install gcc-arm-linux-gnueabihf`

### Build for x86_64 (native)

```bash
cargo build --release --features mqtt
```

Binary: `target/release/time-api`

### Build for ARM (Raspberry Pi)

```bash
# Install ARM target
rustup target add armv7-unknown-linux-gnueabihf

# Build
cargo build --release --target armv7-unknown-linux-gnueabihf --features mqtt
```

Binary: `target/armv7-unknown-linux-gnueabihf/release/time-api`

### Build for multiple architectures

```bash
./deployment/scripts/build-all.sh
```

This builds both x86_64 and ARM binaries with size optimization.

## Deployment

### Raspberry Pi with systemd

1. **Build for ARM:**

   ```bash
   ./deployment/scripts/build-all.sh
   ```

2. **Copy binary to Raspberry Pi:**

   ```bash
   scp target/armv7-unknown-linux-gnueabihf/release/time-api pi@raspberrypi:/home/pi/time-api-rust/
   ```

3. **Copy systemd files:**

   ```bash
   scp deployment/systemd/time-api.service pi@raspberrypi:/tmp/
   scp deployment/systemd/time-api.env pi@raspberrypi:/tmp/
   ```

4. **On Raspberry Pi, install service:**

   ```bash
   # Create directory
   mkdir -p /home/pi/time-api-rust

   # Install systemd service
   sudo mv /tmp/time-api.service /etc/systemd/system/
   sudo mv /tmp/time-api.env /etc/time-api.env

   # Edit configuration
   sudo nano /etc/time-api.env

   # Reload systemd and start service
   sudo systemctl daemon-reload
   sudo systemctl enable time-api
   sudo systemctl start time-api

   # Check status
   sudo systemctl status time-api
   sudo journalctl -u time-api -f
   ```

### Reverse Proxy Setup (nginx)

```nginx
server {
    listen 80;
    server_name time.example.com;

    location / {
        proxy_pass http://localhost:8463;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# Add TLS with Let's Encrypt using certbot
```

## MQTT Integration

### Setup MQTT Broker (mosquitto)

```bash
sudo apt-get install mosquitto mosquitto-clients
sudo systemctl enable mosquitto
sudo systemctl start mosquitto
```

### Configure Time API

Edit `/etc/time-api.env`:

```bash
MQTT_BROKER=mqtt://localhost:1883
MQTT_BASE_TOPIC=raspi/time
```

Restart service:

```bash
sudo systemctl restart time-api
```

### Subscribe to Topics

```bash
# PPS pulses (every second)
mosquitto_sub -h localhost -t "raspi/time/pps" -v

# Health status (on change, max every 5s)
mosquitto_sub -h localhost -t "raspi/time/health" -v
```

### Example Output

**PPS topic:**

```json
{"unix":1234567890}
```

**Health topic:**

```json
{
  "status": "healthy",
  "timestamp": 1234567890,
  "checks": {
    "system_clock": {"status": "ok"},
    "chrony": {"status": "ok"}
  },
  "time_quality": {
    "stratum": 1,
    "offset_seconds": 0.000000012,
    "reference_id": "PPS",
    "leap_status": "Normal"
  }
}
```

## Performance

### Benchmarks (x86_64, 100 concurrent connections, 60 seconds)

```bash
# Install oha (HTTP load tester)
cargo install oha

# Single timezone
oha -z 60s -c 100 "http://localhost:8463/times?tz=UTC"

# Multiple timezones
oha -z 60s -c 100 "http://localhost:8463/times?tz=UTC,America/New_York,Europe/London,Asia/Tokyo"
```

**Expected Results:**

| Metric | Python/FastAPI | Rust/Axum |
|--------|----------------|-----------|
| Memory (RSS) | ~150 MB | <20 MB |
| Latency (p50) | ~5 ms | <1 ms |
| Latency (p99) | ~20 ms | <5 ms |
| Throughput | ~1k req/s | >10k req/s |
| Startup time | ~2 s | <100 ms |

### Raspberry Pi Performance

Expected performance on Raspberry Pi 4 (4 cores, 1.5GHz):

- **Throughput:** >5k req/s
- **Memory:** <20MB RSS
- **Latency (p99):** <10ms

## Development

### Run locally

```bash
# Without MQTT
cargo run --release

# With MQTT
cargo run --release --features mqtt

# With custom config
PORT=8080 LOG_LEVEL=debug cargo run --release
```

### Run tests

```bash
cargo test --all-features
```

### Check for errors

```bash
cargo clippy --all-features
```

## Troubleshooting

### Chrony unavailable

If you see "degraded" status with "chrony unavailable", install and configure chrony:

```bash
sudo apt-get install chrony
sudo systemctl enable chrony
sudo systemctl start chrony
```

Check chrony status:

```bash
chronyc tracking
```

### MQTT connection failed

Check MQTT broker is running:

```bash
sudo systemctl status mosquitto
```

Test MQTT connection:

```bash
mosquitto_pub -h localhost -t test -m "hello"
mosquitto_sub -h localhost -t test
```

### High memory usage

Check for memory leaks:

```bash
# Monitor memory over time
watch -n 1 'ps aux | grep time-api | grep -v grep'

# Run with memory profiling
RUST_LOG=debug ./time-api
```

## Security

### Systemd Hardening

The systemd service includes security hardening:

- `NoNewPrivileges=yes` - Prevent privilege escalation
- `PrivateTmp=yes` - Private /tmp directory
- `ProtectSystem=strict` - Read-only root filesystem
- `MemoryMax=50M` - Memory limit
- `TasksMax=100` - Process limit

### Network Security

- CORS: `Access-Control-Allow-Origin: *`
- HSTS: `Strict-Transport-Security: max-age=31536000`
- CSP: `default-src 'self'; style-src 'unsafe-inline'`
- X-Content-Type-Options: `nosniff`
- X-Frame-Options: `DENY`

### Best Practices

1. Run behind a reverse proxy (nginx, Caddy)
2. Use systemd security features
3. Set up monitoring and alerting
4. Regularly update dependencies: `cargo update`
5. Use a proper CA-signed certificate (not self-signed)

## License

See original repository: https://github.com/tfoote000/http-time-api

## Contributing

Contributions welcome! Please follow the existing code style and add tests for new features.

## Known Limitations

1. **Integer Unix timestamps:** No sub-second precision in HTTP API (matches original)
2. **MQTT PPS precision:** ~1-10ms jitter due to tokio scheduler (not true hardware PPS)
3. **chronyc parsing:** May break if chronyc output format changes
4. **Max timezones:** 50 per request to prevent abuse

## Future Enhancements

- Sub-second precision (float unix timestamp, microseconds in ISO8601)
- Prometheus /metrics endpoint
- IP-based rate limiting
- Request ID tracing
- True hardware PPS via /dev/pps0 for MQTT publishing
- Automatic Let's Encrypt certificate provisioning
