from __future__ import annotations

import argparse

from neotui import (
    App,
    Button,
    Form,
    FormField,
    HttpAction,
    Label,
    Panel,
    StatusStrip,
    TextBlock,
    TextInput,
    VBox,
    check,
    run,
)


def build_app() -> App:
    return App(
        Panel(
            VBox(
                Label("Initial form state"),
                TextInput(
                    form="incident",
                    field="summary",
                    value_from="$forms.incident.summary",
                    placeholder="Describe the incident",
                ),
                TextBlock("The input above is backed by form state."),
                StatusStrip(
                    "idle",
                    status="info",
                    text_from="$actions.submit_incident.$status",
                    status_from="$actions.submit_incident.$status",
                ),
                Button("Submit Incident", on_click="submit_incident"),
                gap=1,
            ),
            title="Form Intent",
        ),
        forms=[
            Form(
                "incident",
                [
                    FormField(
                        "summary",
                        initial="Disk full on db-primary",
                        required=True,
                    )
                ],
            )
        ],
        actions=[
            HttpAction(
                "submit_incident",
                "http://127.0.0.1:7878/ack",
                body={"json": {"summary": "$forms.incident.summary"}},
            )
        ],
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run the NeoTUI Python form intent example.")
    parser.add_argument(
        "--json",
        action="store_true",
        help="print the serialized NeoTUI app JSON and exit",
    )
    parser.add_argument(
        "--check-only",
        action="store_true",
        help="validate the serialized app and exit without launching the runtime",
    )
    parser.add_argument(
        "--neotui-bin",
        help="path to an already-built neotui binary for check validation",
    )
    args = parser.parse_args(argv)

    app = build_app()
    if args.json:
        print(app.to_json())
        return 0

    result = check(app, neotui_bin=args.neotui_bin)
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="")
    if not result.ok or args.check_only:
        return result.returncode
    return run(app).returncode


if __name__ == "__main__":
    raise SystemExit(main())
