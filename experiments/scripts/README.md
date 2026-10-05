# Shared experiment scripts

This directory will contain deterministic helpers reused by more than one `EXP-NNN` experiment.
Each script must accept explicit inputs, print its effective parameters and avoid reading credentials
except through documented environment variables.

An experiment's `commands.sh` remains the complete reproduction entry point. When it invokes a
shared script, its manifest records the project revision containing that script. A shared script
must not overwrite raw results from an earlier run.
