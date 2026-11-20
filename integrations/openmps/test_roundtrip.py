"""Run CPM's contract against a temporary, real PHP OpenMPS server."""
import importlib.util
import os
from pathlib import Path
import subprocess
import sys


def main():
    root = Path(__file__).resolve().parents[2]
    server = Path(os.environ.get("OPENMPS_SOURCE", str(root.parent / "openmps"))).resolve()
    tests = server / "tests/test_api.py"
    if not tests.is_file():
        raise SystemExit("Apply the OpenMPS companion patch, then set OPENMPS_SOURCE to that checkout.")
    spec = importlib.util.spec_from_file_location("mps_fixture", tests)
    fixture = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(fixture)
    fixture.ApiTests.setUpClass()
    try:
        env = os.environ.copy()
        env["CPM_OPENMPS_TEST_URL"] = fixture.ApiTests.url
        env["CPM_OPENMPS_TEST_TOKEN"] = fixture.ApiTests.token
        return subprocess.run(
            ["cargo", "test", "-p", "cpm-app",
             "openmps::client::contract_tests::publishes_and_downloads_against_real_openmps_api",
             "--", "--ignored", "--nocapture"],
            cwd=root, env=env,
        ).returncode
    finally:
        fixture.ApiTests.tearDownClass()


if __name__ == "__main__":
    sys.exit(main())
