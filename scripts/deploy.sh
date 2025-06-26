toolforge jobs delete bot
cp ./target/release/graphbot ~
toolforge jobs run bot --command ~/graphbot --image python3.11 --continuous
