#!/bin/bash

# use bash strict mode
set -euo pipefail

# delete the venv, if it already exists
rm -rf .venv

# create the venv
python3 -m venv .venv

# activate it
source .venv/bin/activate

# upgrade pip inside the venv and add support for the wheel package format
pip install -U pip wheel

pip install -r requirements.txt