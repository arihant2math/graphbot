toolforge jobs delete pyserver
cp ./target/release/graphbot ~
toolforge jobs run pyserver --command ~/graphbot/scripts/run_python.sh --image python3.11 --continuous
