# Deployment Guide

This guide provides step-by-step instructions for deploying the Time API to a Raspberry Pi with chrony/NTP time synchronization.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Build](#build)
3. [Deployment to Raspberry Pi](#deployment-to-raspberry-pi)
4. [Configuration](#configuration)
5. [Monitoring](#monitoring)
6. [Reverse Proxy Setup](#reverse-proxy-setup)
7. [MQTT Setup (Optional)](#mqtt-setup-optional)
8. [Troubleshooting](#troubleshooting)

## Prerequisites

### Development Machine

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- For ARM cross-compilation:
  ```bash
  sudo apt-get install gcc-arm-linux-gnueabihf
  rustup target add armv7-unknown-linux-gnueabihf
  ```

### Raspberry Pi

- Raspberry Pi 3, 4, or 5 (any model with network)
- Raspberry Pi OS (formerly Raspbian) - Debian 11 (Bullseye) or newer
- chrony or NTP for time synchronization
- SSH access enabled

## Build

### Option 1: Build for ARM on Development Machine (Recommended)

```bash
# Clone repository
git clone https://github.com/tfoote000/http-time-api.git
cd http-time-api

# Build for ARM
./deployment/scripts/build-all.sh

# Binary will be at: target/armv7-unknown-linux-gnueabihf/release/time-api
```

### Option 2: Build Natively on Raspberry Pi

```bash
# On Raspberry Pi
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Clone and build
git clone https://github.com/tfoote000/http-time-api.git
cd http-time-api
cargo build --release --features mqtt

# Binary will be at: target/release/time-api
```

**Note:** Building on Raspberry Pi takes significantly longer (20-30 minutes vs 5 minutes on x86_64).

## Deployment to Raspberry Pi

### 1. Create Deployment Directory

On Raspberry Pi:

```bash
mkdir -p ~/time-api-rust
cd ~/time-api-rust
```

### 2. Copy Binary

From development machine:

```bash
# Copy binary
scp target/armv7-unknown-linux-gnueabihf/release/time-api pi@raspberrypi:~/time-api-rust/

# Make executable
ssh pi@raspberrypi "chmod +x ~/time-api-rust/time-api"
```

### 3. Test Binary

On Raspberry Pi:

```bash
cd ~/time-api-rust
./time-api &

# Test endpoint
curl http://localhost:8463/times?tz=UTC

# Stop test
killall time-api
```

### 4. Install Systemd Service

From development machine:

```bash
# Copy systemd files
scp deployment/systemd/time-api.service pi@raspberrypi:/tmp/
scp deployment/systemd/time-api.env pi@raspberrypi:/tmp/
```

On Raspberry Pi:

```bash
# Install service
sudo mv /tmp/time-api.service /etc/systemd/system/
sudo mv /tmp/time-api.env /etc/time-api.env

# Reload systemd
sudo systemctl daemon-reload

# Enable and start service
sudo systemctl enable time-api
sudo systemctl start time-api

# Check status
sudo systemctl status time-api
```

## Configuration

### Edit Configuration

On Raspberry Pi:

```bash
sudo nano /etc/time-api.env
```

### Basic Configuration (HTTP only)

```bash
# HTTP Server
PORT=8463
HOST=0.0.0.0
LOG_LEVEL=info
```

### With MQTT

```bash
# HTTP Server
PORT=8463
HOST=0.0.0.0
LOG_LEVEL=info

# MQTT
MQTT_BROKER=mqtt://localhost:1883
MQTT_BASE_TOPIC=raspi/time
```

### Apply Configuration Changes

```bash
sudo systemctl restart time-api
```

## Monitoring

### View Logs

```bash
# Follow logs in real-time
sudo journalctl -u time-api -f

# View last 100 lines
sudo journalctl -u time-api -n 100

# View logs since boot
sudo journalctl -u time-api -b
```

### Check Service Status

```bash
# Service status
sudo systemctl status time-api

# Is service running?
systemctl is-active time-api

# Is service enabled?
systemctl is-enabled time-api
```

### Monitor Performance

```bash
# CPU and memory usage
top -p $(pgrep time-api)

# Memory usage over time
watch -n 1 'ps aux | grep time-api | grep -v grep'
```

### Health Check

```bash
# Basic health check
curl http://localhost:8463/health | jq

# Ready check
curl http://localhost:8463/ready
```

## Reverse Proxy Setup

### nginx

Install nginx:

```bash
sudo apt-get update
sudo apt-get install nginx
```

Create configuration:

```bash
sudo nano /etc/nginx/sites-available/time-api
```

Add:

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

        # CORS headers (if needed)
        add_header Access-Control-Allow-Origin *;
    }
}
```

Enable and reload:

```bash
sudo ln -s /etc/nginx/sites-available/time-api /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### Add HTTPS with Let's Encrypt

```bash
sudo apt-get install certbot python3-certbot-nginx
sudo certbot --nginx -d time.example.com
```

### Caddy (Alternative)

Install Caddy:

```bash
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy
```

Create Caddyfile:

```bash
sudo nano /etc/caddy/Caddyfile
```

Add:

```
time.example.com {
    reverse_proxy localhost:8463
}
```

Reload:

```bash
sudo systemctl reload caddy
```

## MQTT Setup (Optional)

### Install Mosquitto

```bash
sudo apt-get update
sudo apt-get install mosquitto mosquitto-clients
sudo systemctl enable mosquitto
sudo systemctl start mosquitto
```

### Configure Authentication (Optional)

```bash
# Create password file
sudo mosquitto_passwd -c /etc/mosquitto/passwd timeapi

# Edit mosquitto config
sudo nano /etc/mosquitto/mosquitto.conf
```

Add:

```
allow_anonymous false
password_file /etc/mosquitto/passwd
```

Restart:

```bash
sudo systemctl restart mosquitto
```

### Configure Time API for MQTT

```bash
sudo nano /etc/time-api.env
```

Add:

```bash
MQTT_BROKER=mqtt://localhost:1883
MQTT_USERNAME=timeapi
MQTT_PASSWORD=your_password_here
MQTT_BASE_TOPIC=raspi/time
```

Restart Time API:

```bash
sudo systemctl restart time-api
```

### Test MQTT

Subscribe to topics:

```bash
# PPS (every second)
mosquitto_sub -h localhost -u timeapi -P your_password_here -t "raspi/time/pps" -v

# Health (on change)
mosquitto_sub -h localhost -u timeapi -P your_password_here -t "raspi/time/health" -v
```

## Troubleshooting

### Service won't start

Check logs:

```bash
sudo journalctl -u time-api -n 50
```

Common issues:
- Port already in use: Check with `sudo netstat -tulpn | grep 8463`
- Permission denied: Ensure binary is executable and user has permissions
- Binary not found: Check binary path in systemd service file

### Degraded health status

Check chrony:

```bash
chronyc tracking
systemctl status chrony
```

Install/fix chrony:

```bash
sudo apt-get install chrony
sudo systemctl enable chrony
sudo systemctl start chrony
```

### MQTT connection failed

Check mosquitto:

```bash
systemctl status mosquitto
mosquitto_sub -h localhost -t test
```

Test MQTT config in Time API:

```bash
# View logs for MQTT errors
sudo journalctl -u time-api | grep -i mqtt
```

### High memory usage

Check for leaks:

```bash
# Monitor over time
watch -n 5 'systemctl status time-api | grep Memory'

# Check systemd limits
systemctl show time-api | grep Memory
```

### Verify API responses

```bash
# Test all endpoints
curl http://localhost:8463/ | head -20
curl http://localhost:8463/times?tz=UTC | jq
curl http://localhost:8463/times?tz=UTC,America/Denver,Europe/London | jq
curl http://localhost:8463/health | jq
curl http://localhost:8463/ready
```

### Performance testing

Install oha:

```bash
cargo install oha
```

Run load test:

```bash
# 100 concurrent requests for 60 seconds
oha -z 60s -c 100 "http://localhost:8463/times?tz=UTC"

# Monitor during test
watch -n 1 'ps aux | grep time-api | grep -v grep'
```

## Firewall Configuration

If using ufw:

```bash
# Allow HTTP
sudo ufw allow 80/tcp

# Allow HTTPS
sudo ufw allow 443/tcp

# If accessing Time API directly (not recommended)
sudo ufw allow 8463/tcp

# Enable firewall
sudo ufw enable
```

## Updates

### Update binary

```bash
# Build new version on dev machine
./deployment/scripts/build-all.sh

# Copy to Raspberry Pi
scp target/armv7-unknown-linux-gnueabihf/release/time-api pi@raspberrypi:~/time-api-rust/

# Restart service
ssh pi@raspberrypi "sudo systemctl restart time-api"
```

### Update configuration

```bash
# Edit config
ssh pi@raspberrypi "sudo nano /etc/time-api.env"

# Restart
ssh pi@raspberrypi "sudo systemctl restart time-api"
```

## Production Checklist

- [ ] Raspberry Pi has stable power supply
- [ ] chrony is installed and synchronized
- [ ] Time API systemd service is enabled and running
- [ ] Reverse proxy (nginx/Caddy) is configured with HTTPS
- [ ] Firewall is configured (only 80/443 exposed)
- [ ] MQTT broker has authentication enabled (if using MQTT)
- [ ] Monitoring/alerting is set up
- [ ] Logs are being rotated (systemd handles this automatically)
- [ ] Backup strategy for configuration files

## Additional Resources

- Original Python version: https://github.com/tfoote000/http-time-api
- chrony documentation: https://chrony.tuxfamily.org/
- nginx documentation: https://nginx.org/en/docs/
- Caddy documentation: https://caddyserver.com/docs/
- mosquitto documentation: https://mosquitto.org/documentation/
