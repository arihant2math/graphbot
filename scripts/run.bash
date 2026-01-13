#!/usr/bin/env bash
source ~/.profile
cd ~/graphbot
source .venv/bin/activate
python3 server.py &
cargo build --release -p frontend
cargo build --release
mkdir -p ~/bin/
cp ./target/release/graphbot ~/bin/
./target/release/graphbot
