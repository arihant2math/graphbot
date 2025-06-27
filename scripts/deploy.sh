toolforge jobs delete bot
mkdir -p ~/bin/
cp ./target/release/graphbot ~/bin/
toolforge jobs run bot --command ~/bin/graphbot --image python3.11 --continuous
