toolforge jobs delete bot
mkdir -p ~/bin/
cp ./target/release/graphbot ~/bin/
toolforge jobs run bot --command "bash -c 'source ~/.profile && cd ~/graphbot && & bash ./scripts/run_python.bash && cargo run -r'" --image python3.11 --continuous --cpu 3 --mem 2G
