# carp-cli

Read [Copenhagen Research Platform][carp] study data from Python.

This is the same client the `carp` command line uses, exposed as a module, so
the two share a session: sign in once and either can use it.

```sh
pip install carp-cli
pip install 'carp-cli[pandas]'   # adds to_pandas()
```

> Installed as `carp-cli`, imported as `carp`. PyPI's `carp` was claimed in
> 2012 by an unrelated package that has not been touched since; a distribution
> name has never had to match what you import.

```python
import carp

client = carp.Client(env="production")
client.login()                       # opens a browser, once

for study in client.studies():
    print(study["studyId"], study["name"])
```

## Getting data out

`data_stream` returns one dictionary per measurement, with the stream it came
from folded into each — the shape a DataFrame wants:

```python
rows = client.data_stream(
    deployment="df98d925-3ab4-4b78-8139-fea86d809dc5",
    device="Primary Phone",
    data_type="dk.cachet.carp.heartrate",
    start="7d",
)

frame = carp.to_pandas(rows)         # pip install 'carp-cli[pandas]'
```

Windows are written the way you would say them — `"7d"`, `"36h"`,
`"2026-08-01"`, `"2026-08-01T09:30:00Z"`, or `None` for now.

For the bulk of a study, ask for an export and download the archive:

```python
client.create_export(STUDY)                       # packaged in the background
[export] = [e for e in client.exports(STUDY) if e["status"] == "AVAILABLE"]
path = client.download_export(STUDY, export["id"])
```

## What it returns

Plain lists and dictionaries, exactly as CARP sent them. Nothing is dropped
when the server grows a field, and nothing needs a new release here to become
visible. Keys are camelCase, as on the wire.

`data_stream_raw` returns the server's own nested response instead of the flat
rows — worth reaching for if a measurement type looks wrong, since CARP's
OpenAPI document does not describe the measurement payload.

## Sessions

The session lives in the same file `carp auth login` writes, keyed by the
deployment's host. So:

- `carp auth login --env test` in a terminal, then `carp.Client(env="test")`
  in a notebook, needs no second login
- `client.login()` in a notebook signs the terminal in too
- each deployment keeps its own session; moving between them signs you out of
  neither

## Exceptions

| | |
| --- | --- |
| `carp.CarpAuthError` | no session, or the server rejected it — call `login()` |
| `carp.CarpNotFoundError` | no such study, deployment, export or file |
| `carp.CarpForbiddenError` | signed in, but not allowed to see it |
| `carp.CarpError` | anything else; the base of the three above |

## Building it

```sh
pip install maturin
maturin develop            # into the active virtualenv
maturin build --release    # a wheel, in target/wheels/
```

[carp]: https://carp.dk
