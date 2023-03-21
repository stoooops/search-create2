# CREATE2 Address Searcher

This is a simple tool to search for CREATE2 addresses.

## Usage
```bash
run.sh \
    --factory <CREATE2 factory address> \
    --deployer <msg.sender address> \
    --salt <salt> \
    --init-code-hash <init code hash> \
    --zeros <number of leading zeros to search for> \
    --round-size <round size> \
    --num-rounds <number of rounds> \
    --num-threads <number of threads> \
```