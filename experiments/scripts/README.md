# Shared experiment scripts

This directory contains deterministic experiment helpers, starting with the Day 4 corpus collector.
Each script must accept explicit inputs, print its effective parameters and avoid reading credentials
except through documented environment variables.

An experiment's `commands.sh` remains the complete reproduction entry point. When it invokes a
shared script, its manifest records the project revision containing that script. A shared script
must not overwrite raw results from an earlier run.

`collect_corpus.py` uses Python's standard library and the existing `cast` command. Its commands
are `select-addresses`, `collect` and `audit`; run any with `--help` for explicit arguments.
`collect` reads a public HTTP(S) endpoint from `PVRPC_RPC_URL`. Credentialed/query URLs are rejected
and only the endpoint origin is retained. Account/storage proof verification is not implemented.

```bash
python3 -m unittest discover -s experiments/scripts/tests -v
python3 experiments/scripts/collect_corpus.py audit experiments/EXP-003-real-proof-corpus/retained
```
