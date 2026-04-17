"""Fleet blueprint — operational dashboard for termlink fleet health (T-1103)."""

import json
import os
import subprocess

from flask import Blueprint, jsonify

from web.shared import render_page

bp = Blueprint("fleet", __name__)


def _find_termlink():
    """Find the termlink binary, checking common locations."""
    import shutil
    # Try PATH first
    found = shutil.which("termlink")
    if found:
        return found
    # Check common install locations
    for path in ["/root/.cargo/bin/termlink", "/usr/local/bin/termlink"]:
        if os.path.isfile(path) and os.access(path, os.X_OK):
            return path
    return "termlink"  # fallback to PATH


def _get_fleet_status():
    """Run termlink fleet status --json and return parsed result."""
    try:
        binary = _find_termlink()
        # Ensure HOME and PATH are set so termlink can find hubs.toml and libs
        env = os.environ.copy()
        env.setdefault("HOME", "/root")
        # Add cargo bin to PATH for subprocess
        cargo_bin = os.path.expanduser("~/.cargo/bin")
        if cargo_bin not in env.get("PATH", ""):
            env["PATH"] = cargo_bin + ":" + env.get("PATH", "/usr/bin")
        result = subprocess.run(
            [binary, "fleet", "status", "--json", "--timeout", "5"],
            capture_output=True, text=True, timeout=30, env=env,
        )
        if result.stdout.strip():
            return json.loads(result.stdout)
        # If stdout is empty, check stderr for clues
        if result.stderr.strip():
            return {
                "ok": False, "fleet": [], "actions": [],
                "summary": {"total": 0, "up": 0, "down": 0, "auth_fail": 0},
                "error": result.stderr.strip()[:200],
            }
    except (subprocess.TimeoutExpired, json.JSONDecodeError, FileNotFoundError) as e:
        return {
            "ok": False, "fleet": [], "actions": [],
            "summary": {"total": 0, "up": 0, "down": 0, "auth_fail": 0},
            "error": str(e)[:200],
        }
    return {
        "ok": False, "fleet": [], "actions": [],
        "summary": {"total": 0, "up": 0, "down": 0, "auth_fail": 0},
        "error": "Could not run termlink fleet status",
    }


@bp.route("/fleet")
def fleet_dashboard():
    """Render the fleet operational dashboard."""
    data = _get_fleet_status()
    return render_page(
        "fleet.html",
        page_title="Fleet",
        fleet_data=data,
        fleet_entries=data.get("fleet", []),
        summary=data.get("summary", {}),
        actions=data.get("actions", []),
        is_healthy=data.get("ok", False),
    )


@bp.route("/api/fleet/status")
def fleet_status_api():
    """JSON API endpoint for fleet status (used by auto-refresh)."""
    return jsonify(_get_fleet_status())
