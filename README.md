# GraphBot
Quickly convert old graphs from the graph extension to the chart extension.

## Requirements

- Rust
- Python
- uv

## Usage
First start the parsing server:

`uv run server.py`

Then run the bot:
`cargo run`

## Setup on Toolforge
1. `sh scripts/build_python.sh`
2. Setup rust (https://wikitech.wikimedia.org/wiki/Help:Toolforge/Rust)
3. Setup toolsdb (https://wikitech.wikimedia.org/wiki/Help:Toolforge/ToolsDB)
4. Create database for graph task in toolsdb
5. Create `conf/` files.
6. Clone graphbot into `~/graphbot`
7. Run `sh ./scripts/deploy.sh`

## Deployment on Toolforge
Run `sh ./scripts/deploy.sh`.
