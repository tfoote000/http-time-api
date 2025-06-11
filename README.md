# Time API

A tiny FastAPI service that returns:

- **unix**: seconds since the epoch (UTC)
- **zones**: per-IANA time-zone local time & offset

Supports multiple zones in one call:

GET /times?tz=America/Denver,Europe/London

## File Structure

time-api/
├── README.md
├── requirements.txt
├── Dockerfile
├── docker-compose.yml
├── app/
│ └── main.py
└── example/
├── docker/
│ └── docker-compose.yml
└── systemd/
├── time-api.env
└── time-api.service

## Quickstart (Python + venv)

1. **Create and activate venv**

```bash
   python3 -m venv venv
   source venv/bin/activate
   pip install -r requirements.txt
```

2. **Run**

```bash
uvicorn app.main:app --host 0.0.0.0 --port 8463
```

3. **Test**

```bash
curl "http://localhost:8463/times?tz=UTC,America/Denver"
```

## Docker

1. **Build**

```bash
docker build -t time-api:latest .
```

2. **Run**

```bash
docker run -d --name time-api -e PORT=8463 -p 8463:8463 time-api:latest
```

3. **Or compose**

```bash
docker-compose up -d
```

## Systemd (no Docker)

1. Install venv & deps in /home/pi/time-api

2. (Optional) Create /etc/time-api.env to override:

```ini
PORT=8000
```

3. Drop `example/systemd/time-api.service` → `/etc/systemd/system/time-api.service`

4. Enable & start

```bash
sudo systemctl daemon-reload
sudo systemctl enable time-api
sudo systemctl start time-api
sudo journalctl -u time-api -f
```

5. Test

```bash
curl "http://<nano-pi-ip>:8463/times?tz=Asia/Tokyo,Europe/London"
```

## Environment Variables

- PORT (default 8463) — port for UVicorn
- (you can add more in /etc/time-api.env and reference them in the systemd unit)

## License & Contributing

Feel free to fork and adapt! PRs welcome for bug-fixes, tests, or feature enhancements.
