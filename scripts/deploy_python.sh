toolforge jobs delete pyserver
toolforge jobs run pyserver --command "cd $PWD && bash ./scripts/python_run.bash" --image python3.11 --continuous
