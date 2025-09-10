#!/bin/bash
set -e

if [ ! -d .venv ]; then
    python3 -m venv .venv
    source .venv/bin/activate
    pip3 install matplotlib
fi

source .venv/bin/activate
python3 scripts/plot_benches.py