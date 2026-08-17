"""Tests for the binding layer.

None of these touch the network. What they cover is the seam between Python
and Rust — that arguments cross it correctly, that failures arrive as the
exception a caller would catch, and that the session is read from where the
``carp`` command line writes it. Whether CARP itself answers correctly is
tested on the Rust side.
"""

import json
import subprocess
import sys
from datetime import datetime, timedelta, timezone

import pytest

import carp


@pytest.fixture
def data_dir(tmp_path, monkeypatch):
    """An empty CARP data directory, so no real session is read or written."""
    monkeypatch.setenv("CARP_DATA_DIR", str(tmp_path))
    return tmp_path


def signed_in_token(data_dir, host_slug, account="researcher@dtu.dk"):
    """Write a session file of the shape ``carp auth login`` leaves behind."""
    import base64

    payload = base64.urlsafe_b64encode(
        json.dumps({"preferred_username": account}).encode()
    ).rstrip(b"=").decode()
    expires = datetime.now(timezone.utc) + timedelta(hours=1)
    (data_dir / f"tokens-{host_slug}.json").write_text(
        json.dumps(
            {
                "access_token": f"header.{payload}.signature",
                "refresh_token": "r",
                "expires_at": expires.isoformat().replace("+00:00", "Z"),
            }
        )
    )


def test_module_surface():
    assert isinstance(carp.__version__, str) and carp.__version__
    assert set(carp.environments()) == {"dev", "test", "production"}
    for name in carp.__all__:
        assert hasattr(carp, name), name


@pytest.mark.parametrize(
    ("env", "expected"),
    [
        ("production", "https://carp.computerome.dk"),
        ("test", "https://test.carp.dk"),
        ("dev", "https://dev.carp.dk"),
        # The shorthands the CLI accepts work here too.
        ("prod", "https://carp.computerome.dk"),
        ("staging", "https://test.carp.dk"),
    ],
)
def test_a_deployment_can_be_named(env, expected, data_dir):
    assert carp.Client(env=env).server == expected


def test_an_address_outranks_a_name(data_dir):
    client = carp.Client(env="dev", server="https://carp.example.org")
    assert client.server == "https://carp.example.org"


def test_an_unknown_deployment_is_refused(data_dir):
    """A typo must not quietly fall through to production."""
    with pytest.raises(ValueError, match="unknown CARP environment"):
        carp.Client(env="prod-eu")


def test_no_session_reports_as_such(data_dir):
    status = carp.Client(env="test").status()
    assert status == {
        "server": "https://test.carp.dk",
        "signedIn": False,
        "account": None,
    }


def test_a_session_written_by_the_cli_is_picked_up(data_dir):
    """The whole point of sharing a token store: sign in once, use both."""
    signed_in_token(data_dir, "test-carp-dk")

    status = carp.Client(env="test").status()
    assert status["signedIn"] is True
    assert status["account"] == "researcher@dtu.dk"

    # And only for the deployment it was written for.
    assert carp.Client(env="dev").status()["signedIn"] is False


def test_reading_without_a_session_says_to_sign_in(data_dir):
    client = carp.Client(env="test")
    for call in (
        lambda: client.studies(),
        lambda: client.participants("study"),
        lambda: client.deployments("study"),
        lambda: client.exports("study"),
        lambda: client.files("study"),
        lambda: client.data_stream("d", "Primary", "dk.cachet.carp.heartrate"),
    ):
        with pytest.raises(carp.CarpAuthError):
            call()


def test_the_exceptions_form_a_hierarchy():
    """A caller who only wants "something went wrong" catches the base."""
    for specific in (
        carp.CarpAuthError,
        carp.CarpNotFoundError,
        carp.CarpForbiddenError,
    ):
        assert issubclass(specific, carp.CarpError)
    assert issubclass(carp.CarpError, Exception)


@pytest.mark.parametrize(
    ("kwargs", "message"),
    [
        ({"data_type": ""}, "data type cannot be empty"),
        ({"start": "last tuesday"}, "is not a date"),
        # A window that ends before it starts returns nothing, which reads
        # exactly like a study with no data. Better to refuse it.
        ({"start": "1d", "end": "7d"}, "ends before it starts"),
    ],
)
def test_bad_arguments_raise_value_error(kwargs, message, data_dir):
    """A mistake in the call is a ValueError, not a CARP failure."""
    client = carp.Client(env="test")
    call = {
        "deployment": "d",
        "device": "Primary",
        "data_type": "dk.cachet.carp.heartrate",
        "start": "7d",
        **kwargs,
    }
    with pytest.raises(ValueError, match=message):
        client.data_stream(**call)


@pytest.mark.parametrize("window", ["7d", "36h", "90m", "2w", "2026-08-01", "2026-08-01T09:30:00Z"])
def test_every_window_form_is_accepted(window, data_dir):
    """Parsing happens before the request, so an auth error means it parsed."""
    client = carp.Client(env="test")
    with pytest.raises(carp.CarpAuthError):
        client.data_stream("d", "Primary", "dk.cachet.carp.heartrate", start=window)


def test_to_pandas_needs_pandas_only_when_called():
    """pandas is optional, so importing carp must not require it."""
    pandas = pytest.importorskip("pandas")

    rows = [
        {
            "sequenceId": 40,
            "start": "2024-08-12T12:00:00+00:00",
            "end": None,
            "data": {"steps": 812},
        },
        {
            "sequenceId": 41,
            "start": "2024-08-12T13:00:00+00:00",
            "end": "2024-08-12T14:00:00+00:00",
            "data": {"steps": 431},
        },
    ]

    frame = carp.to_pandas(rows)
    assert list(frame["sequenceId"]) == [40, 41]
    # Times arrive as strings and must land as timestamps: a time column that
    # sorts as text is a trap rather than a convenience.
    assert pandas.api.types.is_datetime64_any_dtype(frame["start"])
    assert pandas.api.types.is_datetime64_any_dtype(frame["end"])
    assert carp.to_pandas([]).empty


def test_importing_carp_does_not_import_pandas():
    """A fresh interpreter, so an already-imported pandas cannot mask it."""
    result = subprocess.run(
        [sys.executable, "-c", "import carp, sys; print('pandas' in sys.modules)"],
        capture_output=True,
        text=True,
        check=True,
    )
    assert result.stdout.strip() == "False", result.stdout
