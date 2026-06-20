#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

PYTHON_CMD=""
if command -v python3 >/dev/null 2>&1; then
  PYTHON_CMD="python3"
elif command -v python >/dev/null 2>&1; then
  PYTHON_CMD="python"
else
  echo "Embedded device smoke requires python3 or python on PATH." >&2
  exit 1
fi

BACKEND_LOG="$(mktemp)"
BACKEND_PID=""
BACKEND_PORT="$("${PYTHON_CMD}" - <<'PY'
import socket

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)"
BACKEND_URL="http://127.0.0.1:${BACKEND_PORT}"

cleanup() {
  if [[ -n "${BACKEND_PID}" ]] && kill -0 "${BACKEND_PID}" >/dev/null 2>&1; then
    kill "${BACKEND_PID}" >/dev/null 2>&1 || true
    wait "${BACKEND_PID}" >/dev/null 2>&1 || true
  fi
  rm -f "${BACKEND_LOG}"
}
trap cleanup EXIT

echo "Validating embedded device fixture tests..."
cargo test -p neotui-core --features http parses_embedded_device_control_example
cargo test -p neotui-cli check_file_accepts_dashboard_examples
cargo test -p neotui-cli form_input_updates_action_payload_for_embedded_device_example

echo "Checking embedded device control app..."
cargo run -p neotui-cli -- check examples/embedded-device-control.toml

echo "Starting mock backend..."
NEOTUI_MOCK_PORT="${BACKEND_PORT}" "${PYTHON_CMD}" scripts/mock-http-backend.py >"${BACKEND_LOG}" 2>&1 &
BACKEND_PID=$!

echo "Waiting for /device/status..."
if ! "${PYTHON_CMD}" - "${BACKEND_URL}" <<'PY'
import json
import sys
import time
from urllib.request import urlopen

base_url = sys.argv[1]
for _ in range(30):
    try:
        with urlopen(f"{base_url}/device/status", timeout=1) as response:
            body = json.loads(response.read().decode("utf-8"))
        assert body["summary"].startswith("edge-gateway-07 online")
        assert isinstance(body["interfaces"], list) and body["interfaces"]
        sys.exit(0)
    except Exception:
        time.sleep(0.2)

raise SystemExit(f"mock backend did not become ready at {base_url}")
PY
then
  echo "Mock backend readiness failed. Backend log:" >&2
  cat "${BACKEND_LOG}" >&2
  exit 1
fi

echo "Posting embedded device actions..."
if ! "${PYTHON_CMD}" - "${BACKEND_URL}" <<'PY'
import json
import sys
from urllib.request import Request, urlopen

base_url = sys.argv[1]

def post(path, payload):
    body = json.dumps(payload).encode("utf-8")
    request = Request(
        f"{base_url}{path}",
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urlopen(request, timeout=2) as response:
        received = json.loads(response.read().decode("utf-8"))
    assert received["ok"] is True
    assert received["received"] == payload

post(
    "/device/apply",
    {
        "hostname": "edge-gateway-99",
        "mode": "field-diagnostic",
        "intent": "apply_config",
    },
)
post(
    "/device/restart",
    {
        "hostname": "edge-gateway-99",
        "mode": "field-diagnostic",
        "intent": "restart_agent",
    },
)
PY
then
  echo "Posting device action payloads failed. Backend log:" >&2
  cat "${BACKEND_LOG}" >&2
  exit 1
fi

if ! grep -q 'device action /device/apply' "${BACKEND_LOG}"; then
  echo "Expected /device/apply action was not printed by the backend." >&2
  cat "${BACKEND_LOG}" >&2
  exit 1
fi

if ! grep -q 'edge-gateway-99' "${BACKEND_LOG}"; then
  echo "Expected edited hostname was not printed by the backend." >&2
  cat "${BACKEND_LOG}" >&2
  exit 1
fi

echo "Embedded device automated checks passed."
