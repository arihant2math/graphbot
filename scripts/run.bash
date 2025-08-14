#!/usr/bin/env bash
source ~/.profile
cd ~/graphbot
source .venv/bin/activate
python3 server.py &
cargo build -r
./target/release/graphbot
