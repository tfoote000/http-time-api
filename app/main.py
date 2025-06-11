from fastapi import FastAPI, HTTPException, Query
from zoneinfo import ZoneInfo
from datetime import datetime, timezone
from typing import List

app = FastAPI(title="Time API")

@app.get("/times")
def read_times(
    tz: List[str] = Query(default=["UTC"], description="Comma-separated IANA zones")
):
    now_utc = datetime.now(timezone.utc)
    unix_ts = int(now_utc.timestamp())

    zones_out = {}
    for name in tz:
        try:
            zone = ZoneInfo(name)
        except Exception:
            raise HTTPException(
                status_code=400, detail=f"Unrecognized time zone '{name}'"
            )

        local_dt = now_utc.astimezone(zone)
        local_str = local_dt.replace(tzinfo=None).isoformat(timespec="seconds")
        offset_sec = int(local_dt.utcoffset().total_seconds())

        zones_out[name] = {"local": local_str, "offset": offset_sec}

    return {"unix": unix_ts, "zones": zones_out}
