#!/bin/bash

POSITIONAL=()
CMD_ARGS=""

while [[ $# -gt 0 ]]
do
key="$1"


# struct Args {
#     /// The CREATE2 factory address
#     #[arg(long)]
#     factory: String,

#     /// The EOA deployer which will call the safeCreate2
#     #[arg(long)]
#     deployer: String,

#     /// the init code hash
#     #[arg(long)]
#     init_code_hash: String,

#     /// zeros to search for
#     #[arg(long)]
#     zeros: Option<u8>,

#     /// number of rounds to search
#     /// each round is a block of size = limit
#     /// each round will increment the initial_salt_n by limit
#     /// so the total number of attempts will be limit * num_rounds
#     /// default is 100,000
#     #[arg(long)]
#     num_rounds: Option<u128>,

#     /// number of attempts per round
#     /// default is 1,000,000
#     /// each round will increment the initial_salt_n by round_size
#     /// so the total number of attempts will be round_size * num_rounds
#     #[arg(long)]
#     round_size: Option<u128>,

#     /// number of threads to use
#     /// default is 16
#     #[arg(long)]
#     num_threads: Option<usize>,
# }
case $key in
    --factory)
    FACTORY="$2"
    CMD_ARGS+="--factory ${FACTORY} "
    shift
    shift
    ;;
    --deployer)
    DEPLOYER="$2"
    CMD_ARGS+="--deployer ${DEPLOYER} "
    shift
    shift
    ;;
    --init-code-hash)
    INIT_CODE_HASH="$2"
    CMD_ARGS+="--init-code-hash ${INIT_CODE_HASH} "
    shift
    shift
    ;;
    --want-zeros)
    WANT_ZEROS="$2"
    CMD_ARGS+="--want-zeros ${WANT_ZEROS} "
    shift
    shift
    ;;
    --num-rounds)
    NUM_ROUNDS="$2"
    CMD_ARGS+="--num-rounds ${NUM_ROUNDS} "
    shift
    shift
    ;;
    --round-size)
    ROUND_SIZE="$2"

    shift
    shift
    ;;
    --num-threads)
    NUM_THREADS="$2"
    CMD_ARGS+="--num-threads ${NUM_THREADS} "
    shift
    shift
    ;;
    *)
    POSITIONAL+=("$1")
    CMD_ARGS+="$1 "
    shift
    ;;
esac
done

export CMD_ARGS
docker-compose up --force-recreate
