#!/usr/bin/env python3
"""Shared I/O helpers for small Garnet status reporters."""
from __future__ import annotations

import sys


def configure_utf8_stdout() -> None:
    """Prefer UTF-8 stdout when the host stream supports reconfiguration.

    Windows consoles can still default to cp1252. Several Garnet reporters emit
    Markdown symbols, so this keeps proof reporters from failing before they can
    report the actual project state.
    """
    reconfigure = getattr(sys.stdout, "reconfigure", None)
    if not callable(reconfigure):
        return
    try:
        reconfigure(encoding="utf-8")
    except (OSError, TypeError, ValueError):
        return
