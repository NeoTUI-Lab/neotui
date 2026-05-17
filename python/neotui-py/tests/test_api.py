from __future__ import annotations

import pathlib
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
SRC = ROOT / "src"
WORKSPACE = ROOT.parents[1]

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
    assert summary["workspace_root_found"] is True


def test_app_serializes_nested_components() -> None:
    import neotui

    app = neotui.App(
        neotui.Panel(
            neotui.VBox(
                neotui.Label("Hello NeoTUI", align="center", width=18),
                neotui.Divider(symbol="="),
                neotui.HBox(
                    neotui.Label("API OK"),
                    neotui.Label("Jobs OK"),
                    gap=2,
                    justify="center",
                ),
                gap=1,
                align="center",
            ),
            id="root-panel",
            title="Operations Board",
        ),
        theme="dark",
    )

    spec = app.to_spec()

    assert spec["schema_version"] == "0.1"
    assert spec["theme"] == "dark"
    assert spec["root"]["kind"] == "Panel"
    assert spec["root"]["props"]["title"] == "Operations Board"
    assert spec["root"]["children"][0]["kind"] == "VBox"
    assert spec["root"]["children"][0]["children"][2]["kind"] == "HBox"


def test_python_api_exposes_future_widget_builders() -> None:
    import neotui

    button = neotui.Button("Deploy")
    listing = neotui.List(["api", "jobs"])
    graph = neotui.Graph([1, 2, 3], title="Latency")

    assert button.to_spec()["kind"] == "Button"
    assert listing.to_spec()["props"]["items"] == ["api", "jobs"]
    assert graph.to_spec()["props"]["title"] == "Latency"


def test_run_builds_cli_command_and_cleans_temp_file(monkeypatch) -> None:
    import neotui

    recorded: dict[str, object] = {}

    class DummyCompletedProcess:
        def __init__(self) -> None:
            self.returncode = 0

    def fake_run(command, cwd, text, check):  # noqa: ANN001
        recorded["command"] = command
        recorded["cwd"] = cwd
        recorded["text"] = text
        recorded["check"] = check
        temp_path = pathlib.Path(command[-1])
        recorded["temp_exists_during_run"] = temp_path.exists()
        recorded["temp_payload"] = temp_path.read_text(encoding="utf-8")
        return DummyCompletedProcess()

    monkeypatch.setattr(neotui.subprocess, "run", fake_run)

    app = neotui.App(neotui.Label("Hello from Python"))
    result = neotui.run(app)

    assert result.returncode == 0
    assert recorded["command"][:6] == [
        "cargo",
        "run",
        "-p",
        "neotui-cli",
        "--",
        "run",
    ]
    assert recorded["temp_exists_during_run"] is True
    assert '"kind": "Label"' in recorded["temp_payload"]
    assert recorded["cwd"] == neotui._workspace_root()


def test_loads_json_builds_app_model() -> None:
    import neotui

    app = neotui.loads_json(
        """
        {
          "schema_version": "0.1",
          "theme": "dark",
          "root": {
            "kind": "Panel",
            "props": {"title": "JSON Demo"},
            "children": [{"kind": "Label", "props": {"text": "Hello"}}]
          }
        }
        """
    )

    assert app.theme == "dark"
    assert app.root.kind == "Panel"
    assert app.root.children[0].props["text"] == "Hello"


def test_load_toml_fixture_builds_app_model() -> None:
    import neotui

    app = neotui.load(WORKSPACE / "examples" / "hello.toml")

    assert app.schema_version == "0.1"
    assert app.theme == "minimal"
    assert app.root.kind == "Label"
    assert app.root.props["text"] == "Hello NeoTUI"


def test_load_json_fixture_builds_nested_component_tree() -> None:
    import neotui

    app = neotui.load(WORKSPACE / "examples" / "dashboard.json")

    assert app.root.kind == "Panel"
    assert app.root.props["title"] == "Release Overview"
    assert [child.kind for child in app.root.children] == [
        "Label",
        "Divider",
        "Spacer",
        "Label",
    ]
