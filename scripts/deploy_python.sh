toolforge jobs delete pyserver
toolforge jobs run pyserver --command "cd $PWD && bash ./scripts/run_python.bash" --image python3.11 --continuous --port 8000
