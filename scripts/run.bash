#!/usr/bin/env bash
source ~/.profile
cd ~/graphbot
source .venv/bin/activate
python3 server.py &
cargo build -r
mkdir -p ~/bin/
cp ./target/release/graphbot ~/bin/
./target/release/graphbot
