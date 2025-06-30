toolforge jobs delete bot
mkdir -p ~/bin/
cp ./target/release/graphbot ~/bin/
toolforge jobs run bot --command "" --image python3.11 --continuous --cpu 3 --mem 2G
