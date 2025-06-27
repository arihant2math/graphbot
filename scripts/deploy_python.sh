toolforge jobs delete pyserver
toolforge jobs run pyserver --command "cd $PWD && source .venv/bin/activate && ./scripts/run_python.sh" --image python3.11 --continuous
