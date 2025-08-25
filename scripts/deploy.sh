echo "Deleting old job and logs..."
toolforge jobs delete bot
rm ~/bot.out ~/bot.err
echo "Deploying new job..."
toolforge jobs run bot --command "bash ~/graphbot/scripts/run.bash" --image python3.11 --continuous --cpu 3 --mem 2G
