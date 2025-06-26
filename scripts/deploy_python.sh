toolforge jobs delete run
cp ./target/release/graphbot ~
toolforge jobs run bot --command ~/graphbot/scripts/run_python.sh --image python3.11 --continuous
