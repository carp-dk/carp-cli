"""Read Copenhagen Research Platform study data from Python.

The same client the ``carp`` command line uses, so the two share a session:
sign in once with ``carp auth login`` and this picks it up, or call
:meth:`Client.login` here and the terminal picks that up.

    >>> import carp
    >>> client = carp.Client(env="test")
    >>> client.login()                            # doctest: +SKIP
    >>> for study in client.studies():            # doctest: +SKIP
    ...     print(study["studyId"], study["name"])

Every method returns plain lists and dictionaries, exactly as CARP sent them,
so nothing is dropped when the server grows a field. :func:`to_pandas` turns
any of them into a DataFrame::

    rows = client.data_stream(
        deployment=DEPLOYMENT,
        device="Primary Phone",
        data_type="dk.cachet.carp.heartrate",
        start="7d",
    )
    frame = carp.to_pandas(rows)

Windows are given the way you would say them: ``"7d"``, ``"36h"``,
``"2026-08-01"``, ``"2026-08-01T09:30:00Z"``, or ``None`` for now.
"""

from ._carp import (
    CarpAuthError,
    CarpError,
    CarpForbiddenError,
    CarpNotFoundError,
    Client,
    __version__,
    environments,
)

__all__ = [
    "Client",
    "CarpError",
    "CarpAuthError",
    "CarpForbiddenError",
    "CarpNotFoundError",
    "environments",
    "to_pandas",
    "__version__",
]


def to_pandas(rows, **kwargs):
    """Build a :class:`pandas.DataFrame` from anything this module returns.

    pandas is imported here rather than at module load, so it stays an optional
    dependency: everything else works without it. Install it with
    ``pip install 'carp-cli[pandas]'``.

    Measurement rows carry ``start`` and ``end`` as ISO-8601 strings; both are
    parsed to timestamps when present, since a time column that sorts as text
    is a trap rather than a convenience.
    """
    try:
        import pandas
    except ImportError as error:  # pragma: no cover - depends on the install
        raise ImportError(
            "to_pandas() needs pandas: pip install 'carp-cli[pandas]'"
        ) from error

    frame = pandas.DataFrame(rows, **kwargs)
    for column in ("start", "end"):
        if column in frame.columns:
            frame[column] = pandas.to_datetime(frame[column], utc=True, errors="coerce")
    return frame
