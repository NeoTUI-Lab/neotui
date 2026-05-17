from __future__ import annotations

import pathlib
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
SRC = ROOT / "src"

if str(SRC) not in sys.path:
    sys.path.insert(0, str(SRC))


def test_import_smoke() -> None:
    import neotui

    assert neotui.__version__ == "0.1.0"
    assert neotui.binding_available is False


def test_package_doctor_summary() -> None:
    import neotui

    summary = neotui.doctor()

    assert summary["package"] == "neotui"
    assert summary["version"] == "0.1.0"
    assert summary["binding_available"] is False
