use clap::Parser;
use ethers_core::types::{Address, U256};
use num_format::{Locale, ToFormattedString};

use crate::utils::{bytes32, count_leading_zeroes};

mod search;
mod utils;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The CREATE2 deployer address
    /// e.g. an ImmutableCreate2Factory or a UniSwap Pool Deployer
    #[arg(long)]
    deployer: String,

    /// The EOA sender which will call the safeCreate2
    #[arg(long)]
    sender: String,

    /// the init code hash
    #[arg(long)]
    init_code_hash: String,

    /// zeros to search for
    #[arg(long)]
    zeros: Option<u8>,

    /// number of rounds to search
    /// each round is a block of size = limit
    /// each round will increment the initial_salt_n by limit
    /// so the total number of attempts will be limit * num_rounds
    /// default is 100,000
    #[arg(long)]
    num_rounds: Option<u128>,

    /// number of attempts per round
    /// default is 1,000,000
    /// each round will increment the initial_salt_n by round_size
    /// so the total number of attempts will be round_size * num_rounds
    #[arg(long)]
    round_size: Option<u128>,

    /// number of threads to use
    /// default is 16
    #[arg(long)]
    num_threads: Option<usize>,
}

fn main() {
    let args = Args::parse();

    let deployer: Address = args.deployer.parse().unwrap();
    let sender: Address = args.sender.parse().unwrap();

    // use U256 because it is copyable in struct via #[derive(Copy, Clone)]
    let init_code_hash: U256 = U256::from_str_radix(
        args.init_code_hash
            .parse::<String>()
            .unwrap()
            .trim_start_matches("0x"),
        16,
    )
    .unwrap();

    let zeros: u8 = args.zeros.unwrap_or(12);
    let num_rounds: u128 = args.num_rounds.unwrap_or(100_000);
    let round_size: u128 = args.round_size.unwrap_or(1_000_000);
    let num_threads: usize = args.num_threads.unwrap_or(16);

    let expected_attempts: u128 = 16_u128.pow(zeros as u32);
    println!(
        "Expected attempts for {} zeros: {}",
        zeros,
        expected_attempts.to_formatted_string(&Locale::en)
    );

    // the initial salt should start with 20 bytes matching the sender address
    // 20 bytes is 40 characters
    let first_40_chars_of_sender = format!("{:x}", sender)[..40].to_string();
    let initial_salt_hex = format!("{}000000000000000000000000", first_40_chars_of_sender);
    // 20 bytes is leaves a search space of 12 bytes or 96 bits

    // setup
    let initial_salt_n = U256::from_str_radix(&initial_salt_hex, 16).unwrap();
    let params = search::SearchParams {
        deployer,
        initial_salt_n,
        init_code_hash: init_code_hash.clone(),
        round_size,
        num_rounds,
    };

    let searcher = search::Searcher::new(num_threads);
    let found: search::AddressSalt = searcher.search(params);

    println!("Best:\n");
    println!(
        "{} zeros {:?} salt 0x{}",
        count_leading_zeroes(found.address),
        found.address,
        hex::encode(bytes32(found.salt_n))
    );
}
