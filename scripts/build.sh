toolforge jobs run build --command "bash -c 'source ~/.profile && cd ~/graphbot && cargo build --release'" --image python3.11 --mem 6G --cpu 3
rm ~/build.out ~/build.err
