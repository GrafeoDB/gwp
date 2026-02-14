"""Shared fixtures for gwp-py tests."""

import os
import socket
import subprocess
import sys
import time
from pathlib import Path

import pytest


def _find_free_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def _wait_for_port(port: int, timeout: float = 10.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.5):
                return
        except OSError:
            time.sleep(0.1)
    raise TimeoutError(f"server did not start on port {port}")


@pytest.fixture(scope="session")
def test_server():
    """Start the gwp-test-server and yield the endpoint address."""
    # Look for the binary in the workspace
    repo_root = Path(__file__).parent.parent.parent
    binary = repo_root / "target" / "release" / "gwp-test-server"
    if sys.platform == "win32":
        binary = binary.with_suffix(".exe")

    if not binary.exists():
        pytest.skip(f"gwp-test-server not found at {binary}")

    port = _find_free_port()
    proc = subprocess.Popen(
        [str(binary), str(port)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    try:
        _wait_for_port(port)
        yield f"localhost:{port}"
    finally:
        proc.terminate()
        proc.wait(timeout=5)
