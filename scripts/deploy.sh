toolforge jobs delete run
cp ./target/release/graphbot ~
toolforge jobs run bot --command ~/graphbot --image python3.11 --continuous

