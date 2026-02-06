use axum::response::{Html, IntoResponse};

/// GET / - API documentation endpoint
pub async fn root() -> impl IntoResponse {
    Html(HTML_CONTENT)
}

const HTML_CONTENT: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Time API Documentation</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            max-width: 900px;
            margin: 40px auto;
            padding: 0 20px;
            line-height: 1.6;
            color: #333;
        }
        h1 {
            color: #2c3e50;
            border-bottom: 2px solid #3498db;
            padding-bottom: 10px;
        }
        h2 {
            color: #34495e;
            margin-top: 30px;
        }
        code {
            background-color: #f4f4f4;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: 'Courier New', Courier, monospace;
        }
        pre {
            background-color: #f4f4f4;
            padding: 15px;
            border-radius: 5px;
            overflow-x: auto;
            border-left: 3px solid #3498db;
        }
        pre code {
            background-color: transparent;
            padding: 0;
        }
        .endpoint {
            background-color: #e8f4f8;
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
        }
        .method {
            display: inline-block;
            background-color: #3498db;
            color: white;
            padding: 3px 8px;
            border-radius: 3px;
            font-weight: bold;
            font-size: 0.9em;
        }
        .example {
            margin-top: 10px;
        }
        .note {
            background-color: #fff3cd;
            border-left: 3px solid #ffc107;
            padding: 10px 15px;
            margin: 15px 0;
        }
    </style>
</head>
<body>
    <h1>Time API Documentation</h1>
    <p>High-performance time API with GPS PPS integration and timezone conversion.</p>

    <div class="endpoint">
        <h2><span class="method">GET</span> /times</h2>
        <p>Get current time in one or more timezones.</p>

        <h3>Query Parameters</h3>
        <ul>
            <li><code>tz</code> (optional): Comma-separated list of IANA timezone names. Default: <code>UTC</code></li>
            <li><code>include_quality</code> (optional): Include time quality metrics from chrony. Default: <code>false</code></li>
        </ul>

        <h3>Response Format</h3>
        <pre><code>{
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
}</code></pre>

        <div class="example">
            <h3>Examples</h3>
            <pre><code># Single timezone (UTC)
curl "http://localhost:8463/times"

# Multiple timezones
curl "http://localhost:8463/times?tz=UTC,America/New_York,Europe/London,Asia/Tokyo"

# With time quality metrics
curl "http://localhost:8463/times?tz=UTC&include_quality=true"</code></pre>
        </div>

        <div class="note">
            <strong>Note:</strong> Unix timestamp is in integer seconds. Local time format is ISO8601 without timezone suffix (YYYY-MM-DDTHH:MM:SS). Offset is in seconds from UTC.
        </div>
    </div>

    <div class="endpoint">
        <h2><span class="method">GET</span> /health</h2>
        <p>Check system health and time quality.</p>

        <h3>Response Format</h3>
        <pre><code>{
  "status": "healthy",
  "checks": {
    "system_clock": {
      "status": "ok"
    },
    "chrony": {
      "status": "ok"
    }
  },
  "time_quality": {
    "stratum": 1,
    "offset_seconds": 0.000000012,
    "reference_id": "PPS",
    "leap_status": "Normal"
  }
}</code></pre>

        <div class="example">
            <h3>Example</h3>
            <pre><code># Check health
curl "http://localhost:8463/health"</code></pre>
        </div>

        <div class="note">
            <strong>Status values:</strong>
            <ul>
                <li><code>healthy</code>: All checks passed, stratum &lt; 4</li>
                <li><code>degraded</code>: All checks passed, stratum 4-15</li>
                <li><code>unhealthy</code>: One or more checks failed, or stratum 16 (unsynced)</li>
            </ul>
        </div>
    </div>

    <div class="endpoint">
        <h2><span class="method">GET</span> /ready</h2>
        <p>Liveness check for monitoring systems.</p>
        <p>Returns HTTP 200 if the server is running and can accept requests.</p>

        <div class="example">
            <h3>Example</h3>
            <pre><code># Check if server is ready
curl "http://localhost:8463/ready"</code></pre>
        </div>
    </div>

    <h2>Error Responses</h2>
    <p>Errors return appropriate HTTP status codes with a JSON body:</p>
    <pre><code>{
  "detail": "Unrecognized time zone 'Invalid/Zone'"
}</code></pre>

    <h2>CORS</h2>
    <p>All endpoints support CORS with <code>Access-Control-Allow-Origin: *</code>.</p>

    <h2>Performance</h2>
    <ul>
        <li>Latency: &lt;1ms p50, &lt;5ms p99</li>
        <li>Throughput: &gt;10,000 requests/second</li>
        <li>Memory: &lt;20MB RSS</li>
    </ul>

    <footer style="margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; text-align: center; color: #7f8c8d;">
        <p>Time API v0.1.0 | Built with Rust + Axum</p>
    </footer>
</body>
</html>
"#;
