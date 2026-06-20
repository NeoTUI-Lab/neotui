#!/usr/bin/env python3
"""Tiny local JSON backend for HTTP, form-intent and device-control examples."""

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
import time


START = time.time()


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path != "/status":
            if self.path == "/device/status":
                self.send_device_status()
                return
            self.send_response(404)
            self.end_headers()
            return

        tick = int(time.time() - START)
        body = {
            "summary": f"backend ready | tick {tick}",
            "health": "success" if tick % 5 else "warning",
            "requests": 128 + tick,
            "cpu": 35 + (tick % 45),
            "latency": [18, 21, 19, 24, 22 + (tick % 9), 20 + (tick % 7)],
            "queue": ["ingest", "render", "ship", f"tick-{tick}"],
        }
        payload = json.dumps(body).encode("utf-8")

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def send_device_status(self):
        tick = int(time.time() - START)
        body = {
            "summary": f"edge-gateway-07 online | sensor tick {tick}",
            "health": "warning" if tick % 6 == 0 else "success",
            "uptime": f"{18 + tick // 60}h {tick % 60:02d}m",
            "cpu": 38 + (tick % 34),
            "temperature": 49 + (tick % 18),
            "load_history": [31, 35, 33, 41, 46, 43 + (tick % 10), 39 + (tick % 8)],
            "interfaces": [
                {"name": "eth0", "state": "up", "rx": f"{180 + tick}M", "tx": f"{96 + tick}M"},
                {"name": "wlan0", "state": "standby", "rx": f"{12 + tick % 5}M", "tx": f"{4 + tick % 3}M"},
                {"name": "can0", "state": "up", "rx": f"{840 + tick}", "tx": f"{816 + tick}"},
            ],
            "events": [
                "sensor bus nominal",
                "config checksum verified",
                "agent heartbeat received",
                f"telemetry sample {tick}",
            ],
            "firmware": "2026.05-edge.4",
            "ip": "10.42.0.17",
            "power": "balanced",
        }
        payload = json.dumps(body).encode("utf-8")

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def do_POST(self):
        if self.path not in {"/ack", "/device/apply", "/device/restart"}:
            self.send_response(404)
            self.end_headers()
            return

        length = int(self.headers.get("Content-Length", "0"))
        raw_body = self.rfile.read(length) if length else b""
        try:
            received = json.loads(raw_body.decode("utf-8")) if raw_body else None
        except json.JSONDecodeError:
            received = raw_body.decode("utf-8", errors="replace")

        if self.path == "/ack":
            print(f"ack payload: {json.dumps(received, sort_keys=True)}", flush=True)
        else:
            print(
                f"device action {self.path}: {json.dumps(received, sort_keys=True)}",
                flush=True,
            )

        payload = json.dumps({"ok": True, "received": received}).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, format, *args):
        return


def main():
    port = int(os.environ.get("NEOTUI_MOCK_PORT", "7878"))
    server = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    print(f"NeoTUI mock backend listening on http://127.0.0.1:{port}/status")
    print("POST /ack echoes received JSON payloads for action smoke tests")
    print("GET /device/status exposes embedded-device telemetry")
    print("POST /device/apply and /device/restart echo device action payloads")
    server.serve_forever()


if __name__ == "__main__":
    main()
