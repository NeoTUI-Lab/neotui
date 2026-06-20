#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_NATIVE=0

for arg in "$@"; do
  case "${arg}" in
    --native)
      RUN_NATIVE=1
      ;;
    -h|--help)
      cat <<'HELP'
Usage: ./scripts/test-python.sh [--native]

Runs the NeoTUI Python package contract tests.

Options:
  --native  Also build/install the PyO3 extension with maturin, validate
            the Python app through neotui check, and rerun tests.
HELP
      exit 0
      ;;
    *)
      echo "Unknown argument: ${arg}" >&2
      exit 1
      ;;
  esac
done

cd "${ROOT_DIR}"

PYTHON_CMD=""
if command -v python >/dev/null 2>&1; then
  PYTHON_CMD="python"
elif command -v python3 >/dev/null 2>&1; then
  PYTHON_CMD="python3"
fi

if ! command -v uv >/dev/null 2>&1 && [[ -z "${PYTHON_CMD}" ]]; then
  echo "Python package tests require uv, python, or python3 on PATH." >&2
  exit 1
fi

if ! command -v uv >/dev/null 2>&1 && [[ "${RUN_NATIVE}" -eq 1 ]]; then
  VENV_DIR="${ROOT_DIR}/target/python-test-venv"
  if [[ ! -x "${VENV_DIR}/bin/python" ]]; then
    if ! "${PYTHON_CMD}" -m venv "${VENV_DIR}"; then
      echo "Could not create ${VENV_DIR}." >&2
      echo "Install python3-venv/python3-full, or install uv, then rerun ./scripts/test-python.sh --native." >&2
      exit 1
    fi
  fi
  PYTHON_CMD="${VENV_DIR}/bin/python"
  export VIRTUAL_ENV="${VENV_DIR}"
  export PATH="${VENV_DIR}/bin:${PATH}"
  "${PYTHON_CMD}" -m pip install --upgrade pip maturin pytest tomli
fi

run_pure_tests() {
  if command -v uv >/dev/null 2>&1; then
    uv run --no-project --with pytest --with tomli python -m pytest -p no:cacheprovider python/neotui-py/tests
  else
    PYTHONPATH="${ROOT_DIR}/python/neotui-py/src${PYTHONPATH:+:${PYTHONPATH}}" \
      "${PYTHON_CMD}" -m pytest -p no:cacheprovider python/neotui-py/tests
  fi
}

echo "Running NeoTUI Python package tests..."
run_pure_tests

echo "Checking Python form intent example JSON serialization..."
CONTRACT_JSON="${ROOT_DIR}/examples/python/form-intent.json"
GENERATED_JSON="/tmp/neotui-python-form-intent.json"
if command -v uv >/dev/null 2>&1; then
  PYTHONPATH="${ROOT_DIR}/python/neotui-py/src${PYTHONPATH:+:${PYTHONPATH}}" \
    uv run --no-project --with tomli python examples/python/form_intent.py --json >"${GENERATED_JSON}"
else
  PYTHONPATH="${ROOT_DIR}/python/neotui-py/src${PYTHONPATH:+:${PYTHONPATH}}" \
    "${PYTHON_CMD}" examples/python/form_intent.py --json >"${GENERATED_JSON}"
fi

if ! cmp -s "${GENERATED_JSON}" "${CONTRACT_JSON}"; then
  echo "Python form intent JSON does not match ${CONTRACT_JSON}." >&2
  diff -u "${CONTRACT_JSON}" "${GENERATED_JSON}" >&2 || true
  exit 1
fi

if [[ "${RUN_NATIVE}" -eq 1 ]]; then
  echo "Building NeoTUI Python native extension with maturin..."
  if command -v uv >/dev/null 2>&1; then
    (cd python/neotui-py && uv run maturin develop)
    (cd python/neotui-py && uv run python -m pytest -p no:cacheprovider tests)
  else
    (cd python/neotui-py && "${PYTHON_CMD}" -m maturin develop)
    "${PYTHON_CMD}" -m pytest -p no:cacheprovider python/neotui-py/tests
  fi

  echo "Validating Python form intent app through neotui check..."
  cargo build -p neotui-cli
  PYTHONPATH="${ROOT_DIR}/python/neotui-py/src${PYTHONPATH:+:${PYTHONPATH}}" \
    "${PYTHON_CMD}" examples/python/form_intent.py --check-only --neotui-bin "${ROOT_DIR}/target/debug/neotui"
fi

echo "NeoTUI Python package checks passed."
