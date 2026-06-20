from __future__ import annotations

import importlib.util
import json
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
    assert isinstance(neotui.binding_available, bool)


def test_package_doctor_summary() -> None:
    import neotui

    summary = neotui.doctor()

    assert summary["package"] == "neotui"
    assert summary["version"] == "0.1.0"
    assert summary["binding_available"] is neotui.binding_available
    assert summary["workspace_root_found"] is True
    assert summary["callback_contract_available"] is True
    assert summary["runtime_callback_bridge"] is False


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
    text_input = neotui.TextInput(
        form="incident",
        field="summary",
        value_from="$forms.incident.summary",
    )
    status = neotui.StatusStrip(
        "idle",
        text_from="$actions.submit_incident.$status",
        status_from="$actions.submit_incident.$status",
    )

    assert button.to_spec()["kind"] == "Button"
    assert listing.to_spec()["props"]["items"] == ["api", "jobs"]
    assert graph.to_spec()["props"]["title"] == "Latency"
    assert text_input.to_spec()["props"]["form"] == "incident"
    assert status.to_spec()["props"]["status_from"] == "$actions.submit_incident.$status"


def test_python_api_exposes_rich_widget_builders() -> None:
    import neotui

    metric = neotui.Metric("42", title="Requests", value_from="ops.requests")
    gauge = neotui.Gauge(72, title="CPU", min=0, max=100, value_from="ops.cpu")
    sparkline = neotui.Sparkline([4, 8, 5], values_from="ops.latency", height=2)
    table = neotui.Table(
        [{"key": "name", "title": "Name"}, {"key": "status", "title": "Status"}],
        [{"name": "api", "status": "ok"}],
        rows_from="ops.services",
    )

    assert metric.to_spec()["kind"] == "Metric"
    assert gauge.to_spec()["props"]["max"] == 100
    assert sparkline.to_spec()["props"]["values_from"] == "ops.latency"
    assert table.to_spec()["props"]["columns"][0]["key"] == "name"


def test_button_callback_invokes_successfully() -> None:
    import neotui

    calls: list[str] = []

    def handler() -> str:
        calls.append("clicked")
        return "ok"

    button = neotui.Button("Deploy", id="deploy", on_click=handler)

    assert button.to_spec()["kind"] == "Button"
    assert "click" not in button.to_spec().get("props", {})
    assert button.invoke("click") == "ok"
    assert calls == ["clicked"]


def test_button_action_binding_serializes_to_dsl_props() -> None:
    import neotui

    button = neotui.Button("Submit", id="submit", on_click="submit_incident")

    assert button.has_callbacks() is False
    assert button.to_spec()["props"]["on_click"] == "submit_incident"


def test_button_callback_failure_is_wrapped() -> None:
    import neotui

    def handler() -> None:
        raise ValueError("boom")

    button = neotui.Button("Deploy", id="deploy", on_click=handler)

    try:
        button.invoke("click")
    except neotui.CallbackError as exc:
        assert exc.component_id == "deploy"
        assert exc.component_kind == "Button"
        assert exc.event_name == "click"
        assert "callback `click` failed for component `deploy`" in str(exc)
    else:
        raise AssertionError("callback failure should be wrapped")


def test_app_reports_callback_bindings() -> None:
    import neotui

    app = neotui.App(
        neotui.Panel(
            neotui.Button("Deploy", id="deploy", on_click=lambda: "ok"),
            neotui.Label("Status"),
        )
    )

    assert app.has_callbacks() is True
    assert app.callback_bindings() == {"deploy": ["click"]}


def test_run_builds_cli_command_and_cleans_temp_file(monkeypatch) -> None:
    import neotui

    recorded: dict[str, object] = {}

    class DummyCompletedProcess:
        def __init__(self) -> None:
            self.returncode = 0

    def fake_run(command, cwd, text, check, capture_output):  # noqa: ANN001
        recorded["command"] = command
        recorded["cwd"] = cwd
        recorded["text"] = text
        recorded["check"] = check
        recorded["capture_output"] = capture_output
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
    assert recorded["capture_output"] is False


def test_check_builds_cli_command_captures_output_and_cleans_temp_file(monkeypatch) -> None:
    import neotui

    recorded: dict[str, object] = {}

    class DummyCompletedProcess:
        def __init__(self) -> None:
            self.returncode = 0
            self.stdout = "check ok\n"
            self.stderr = ""

    def fake_run(command, cwd, text, check, capture_output):  # noqa: ANN001
        recorded["command"] = command
        recorded["cwd"] = cwd
        recorded["text"] = text
        recorded["check"] = check
        recorded["capture_output"] = capture_output
        temp_path = pathlib.Path(command[-1])
        recorded["temp_exists_during_run"] = temp_path.exists()
        recorded["temp_payload"] = temp_path.read_text(encoding="utf-8")
        recorded["temp_path"] = temp_path
        return DummyCompletedProcess()

    monkeypatch.setattr(neotui.subprocess, "run", fake_run)

    app = neotui.App(neotui.Label("Check me"))
    result = neotui.check(app, neotui_bin="neotui")

    assert result.ok is True
    assert result.stdout == "check ok\n"
    assert recorded["command"][:2] == ["neotui", "check"]
    assert recorded["temp_exists_during_run"] is True
    assert recorded["temp_path"].exists() is False
    assert '"text": "Check me"' in recorded["temp_payload"]
    assert recorded["cwd"] == neotui._workspace_root()
    assert recorded["capture_output"] is True


def test_check_returns_failure_without_raising(monkeypatch) -> None:
    import neotui

    class DummyCompletedProcess:
        def __init__(self) -> None:
            self.returncode = 1
            self.stdout = ""
            self.stderr = "invalid component\n"

    def fake_run(command, cwd, text, check, capture_output):  # noqa: ANN001
        return DummyCompletedProcess()

    monkeypatch.setattr(neotui.subprocess, "run", fake_run)

    result = neotui.check(neotui.App(neotui.Component("Nope")))

    assert result.ok is False
    assert result.returncode == 1
    assert result.stderr == "invalid component\n"


def test_run_rejects_python_callbacks_until_runtime_bridge_exists() -> None:
    import neotui

    app = neotui.App(neotui.Button("Deploy", on_click=lambda: "ok"))

    try:
        neotui.run(app)
    except RuntimeError as exc:
        assert "runtime callback bridge is not implemented yet" in str(exc)
    else:
        raise AssertionError("run(app) should reject Python callbacks for now")


def test_app_serializes_forms_actions_and_data_sources() -> None:
    import neotui

    app = neotui.App(
        neotui.Panel(
            neotui.VBox(
                neotui.TextInput(
                    form="incident",
                    field="summary",
                    value_from="$forms.incident.summary",
                ),
                neotui.Button("Submit", on_click="submit_incident"),
            )
        ),
        forms=[
            neotui.Form(
                "incident",
                [
                    neotui.FormField(
                        "summary",
                        initial="Disk full on db-primary",
                        required=True,
                    )
                ],
            )
        ],
        actions=[
            neotui.HttpAction(
                "submit_incident",
                "http://127.0.0.1:7878/ack",
                body={"json": {"summary": "$forms.incident.summary"}},
            )
        ],
        data_sources=[
            neotui.HttpDataSource(
                "ops",
                "http://127.0.0.1:7878/status",
                refresh_ms=5000,
                retry_count=2,
            )
        ],
    )

    spec = app.to_spec()

    assert spec["forms"][0]["fields"][0]["required"] is True
    assert spec["actions"][0]["body"]["json"]["summary"] == "$forms.incident.summary"
    assert spec["data"]["sources"][0]["refresh_ms"] == 5000
    assert spec["root"]["children"][0]["children"][1]["props"]["on_click"] == "submit_incident"


def test_app_roundtrips_forms_actions_and_data_sources() -> None:
    import neotui

    app = neotui.App.from_spec(
        {
            "schema_version": "0.1",
            "theme": "minimal",
            "forms": [
                {
                    "id": "incident",
                    "fields": [
                        {
                            "id": "summary",
                            "kind": "text",
                            "initial": "Disk full",
                            "required": True,
                        }
                    ],
                }
            ],
            "actions": [
                {
                    "id": "submit_incident",
                    "kind": "http",
                    "url": "http://127.0.0.1:7878/ack",
                    "method": "POST",
                    "body": {"json": {"summary": "$forms.incident.summary"}},
                }
            ],
            "data": {
                "sources": [
                    {
                        "id": "ops",
                        "kind": "http",
                        "url": "http://127.0.0.1:7878/status",
                        "refresh_ms": 5000,
                    }
                ]
            },
            "root": {"kind": "Label", "props": {"text": "OK"}},
        }
    )

    assert app.forms[0].fields[0].required is True
    assert app.actions[0].body["json"]["summary"] == "$forms.incident.summary"
    assert app.data_sources[0].refresh_ms == 5000
    assert app.to_spec()["root"]["kind"] == "Label"


def test_python_form_intent_example_serializes_current_dsl() -> None:
    module_path = WORKSPACE / "examples" / "python" / "form_intent.py"
    spec = importlib.util.spec_from_file_location("form_intent_example", module_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    app = module.build_app()
    payload = app.to_spec()

    assert payload["forms"][0]["id"] == "incident"
    assert payload["actions"][0]["body"]["json"]["summary"] == "$forms.incident.summary"
    assert payload["root"]["children"][0]["children"][1]["kind"] == "TextInput"
    assert payload["root"]["children"][0]["children"][4]["props"]["on_click"] == "submit_incident"


def test_python_form_intent_example_json_mode(capsys, monkeypatch) -> None:
    module_path = WORKSPACE / "examples" / "python" / "form_intent.py"
    spec = importlib.util.spec_from_file_location("form_intent_example_json", module_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    def fail_if_called(*args, **kwargs):  # noqa: ANN002, ANN003
        raise AssertionError("JSON mode should not invoke subprocess")

    monkeypatch.setattr(module, "check", fail_if_called)
    monkeypatch.setattr(module, "run", fail_if_called)

    assert module.main(["--json"]) == 0
    payload = json.loads(capsys.readouterr().out)

    assert payload["forms"][0]["fields"][0]["id"] == "summary"
    assert payload["actions"][0]["id"] == "submit_incident"


def test_python_form_intent_example_matches_json_contract(capsys, monkeypatch) -> None:
    module_path = WORKSPACE / "examples" / "python" / "form_intent.py"
    contract_path = WORKSPACE / "examples" / "python" / "form-intent.json"
    spec = importlib.util.spec_from_file_location("form_intent_example_contract", module_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    def fail_if_called(*args, **kwargs):  # noqa: ANN002, ANN003
        raise AssertionError("JSON mode should not invoke subprocess")

    monkeypatch.setattr(module, "check", fail_if_called)
    monkeypatch.setattr(module, "run", fail_if_called)

    assert module.main(["--json"]) == 0
    generated = json.loads(capsys.readouterr().out)
    contract = json.loads(contract_path.read_text(encoding="utf-8"))

    assert generated == contract


def test_python_form_intent_example_check_only(capsys, monkeypatch) -> None:
    module_path = WORKSPACE / "examples" / "python" / "form_intent.py"
    spec = importlib.util.spec_from_file_location("form_intent_example_check", module_path)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    class DummyCheckResult:
        ok = True
        returncode = 0
        stdout = "check ok\n"
        stderr = ""

    recorded: dict[str, object] = {}

    def fake_check(app, neotui_bin=None):  # noqa: ANN001
        recorded["app"] = app
        recorded["neotui_bin"] = neotui_bin
        return DummyCheckResult()

    def fail_if_run(*args, **kwargs):  # noqa: ANN002, ANN003
        raise AssertionError("check-only mode should not run the app")

    monkeypatch.setattr(module, "check", fake_check)
    monkeypatch.setattr(module, "run", fail_if_run)

    assert module.main(["--check-only", "--neotui-bin", "target/debug/neotui"]) == 0
    assert recorded["neotui_bin"] == "target/debug/neotui"
    assert "check ok" in capsys.readouterr().out


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
