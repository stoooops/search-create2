use clap::Parser;
use ethers_core::{
    types::{Address, Bytes, U256},
    utils::get_create2_address_from_hash,
};
use num_format::{Locale, ToFormattedString};
use rayon::{prelude::*, ThreadPoolBuilder};
use std::sync::{Arc, Mutex};

mod utils;

fn log_attempts(round: u128, attempt: u128, now: std::time::Instant, best_zeros: u8) {
    // should be at least 1 to avoid divide by zero
    let elapsed_ms = now.elapsed().as_millis();
    if elapsed_ms == 0 {
        return;
    }

    // calculate the rate of attempts per second
    let rate_ms: f64 = (attempt as f64) / (elapsed_ms as f64);
    let rate = rate_ms * 1000.0;
    println!(
        "Round {} @ {} attempts/sec {}",
        round,
        (rate as u128).to_formatted_string(&Locale::en),
        utils::countdown(best_zeros + 1, rate, elapsed_ms)
    );
}

#[derive(Copy, Clone)]
struct AddressSalt {
    address: Address,
    // leading_zeros: u8,
    salt_n: U256,
}

fn log_best(best: &AddressSalt) {
    let msg = format!(
        "{} zeros {:?} salt 0x{}",
        utils::count_leading_zeroes(best.address),
        best.address,
        hex::encode(utils::bytes32(best.salt_n))
    );
    // print to terminal in cyan ANSI color
    println!("\x1b[36m{}\x1b[0m", msg);
}

fn log_new_best(best: &AddressSalt) {
    let msg = format!(
        "{} zeros {:?} salt 0x{}",
        utils::count_leading_zeroes(best.address),
        best.address,
        hex::encode(utils::bytes32(best.salt_n))
    );
    // print to terminal in green ANSI color
    println!("\x1b[32m{}\x1b[0m", msg);
}

#[derive(Copy, Clone)]
struct SearchParams {
    deployer: Address,
    initial_salt_n: U256,
    init_code_hash: U256,
    limit: u128,
}

fn search_create2_addresses(params: &SearchParams) -> AddressSalt {
    let SearchParams {
        deployer,
        initial_salt_n,
        init_code_hash,
        limit,
    } = params;
    let mut salt_n = *initial_salt_n;
    let mut salt = utils::bytes32(salt_n);

    let init_code_hash_bytes: Bytes = utils::bytes32(*init_code_hash);

    let mut address: Address =
        get_create2_address_from_hash(*deployer, &salt, &init_code_hash_bytes);

    let mut best: AddressSalt = AddressSalt {
        address: address,
        // leading_zeros: address.leading_zeros,
        salt_n,
    };

    // let mut max_zeroes = 0;
    for _i in 0..*limit {
        salt_n += U256::from(1);
        salt = utils::bytes32(salt_n);
        address = get_create2_address_from_hash(*deployer, &salt, &init_code_hash_bytes);
        // check if we have a new best
        if address < best.address {
            best = AddressSalt { address, salt_n };
        }
    }
    return best;
}

fn search_round(
    global_best: &Arc<Mutex<AddressSalt>>,
    total_attempts: &Arc<Mutex<u128>>,
    total_rounds: &Arc<Mutex<u128>>,
    round: u128,
    initial_params: &SearchParams,
    start_time: std::time::Instant,
) -> AddressSalt {
    let SearchParams {
        deployer,
        initial_salt_n,
        init_code_hash,
        limit,
    } = initial_params;

    let round_size = *limit;

    let round_offset = U256::from(*limit) * U256::from(round);
    let round_salt_n = initial_salt_n + round_offset;
    // let round_salt = bytes32(round_salt_n);
    let params = SearchParams {
        deployer: *deployer,
        initial_salt_n: round_salt_n,
        init_code_hash: init_code_hash.clone(),
        limit: round_size,
    };

    let round_best = search_create2_addresses(&params);
    // acquire best mutex and check if there are more leading zeros
    let mut the_best = global_best.lock().unwrap();
    let mut total_rounds = total_rounds.lock().unwrap();
    *total_rounds += 1;
    let mut total_attempts = total_attempts.lock().unwrap();
    *total_attempts += round_size;
    // this will be unlocked when the lock goes out of scope which is when the function returns

    // update best
    if round_best.address < the_best.address {
        // update the best
        *the_best = round_best;
        log_new_best(&the_best);
    } else if *total_rounds % 100 == 0 {
        // periodically log the best
        log_best(&the_best);
    }

    log_attempts(
        *total_rounds,
        *total_attempts,
        start_time,
        utils::count_leading_zeroes(the_best.address),
    );
    return round_best;
}

fn search(
    best: &Arc<Mutex<AddressSalt>>,
    total_attempts: &Arc<Mutex<u128>>,
    total_rounds: &Arc<Mutex<u128>>,
    initial_params: &SearchParams,
    num_rounds: u128,
    num_threads: usize,
) {
    // repeat search in blocks of size = limit, incrementing the inital_salt_n
    let start_time = std::time::Instant::now();

    // Create a custom thread pool with the specified number of threads
    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("Failed to create thread pool");

    // Run the parallel iterator within the context of the custom thread pool
    thread_pool.install(|| {
        (0..num_rounds).into_par_iter().for_each(|round| {
            search_round(
                best,
                total_attempts,
                total_rounds,
                round,
                initial_params,
                start_time,
            );
        });
    });
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The CREATE2 factory address
    #[arg(long)]
    factory: String,

    /// The EOA deployer which will call the safeCreate2
    #[arg(long)]
    deployer: String,

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

    // default: 0x5FbDB2315678afecb367f032d93F642f64180aa3
    let factory: Address = args.factory.parse().unwrap();

    // default: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
    let deployer: Address = args.deployer.parse().unwrap();

    // "5943414e6e6c56bb59082294e78590adbb8e2d6253a2a8d7e43c46afcf5f7012"
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

    // the initial salt should start with 20 bytes matching the deployer address
    // 20 bytes is 40 characters
    let first_40_chars_of_deployer = format!("{:x}", deployer)[..40].to_string();
    let initial_salt_hex = format!("{}000000000000000000000000", first_40_chars_of_deployer);
    // 20 bytes is leaves a search space of 12 bytes or 96 bits

    // setup
    let initial_salt_n = U256::from_str_radix(&initial_salt_hex, 16).unwrap();
    let params = SearchParams {
        deployer: factory,
        initial_salt_n,
        init_code_hash: init_code_hash.clone(),
        limit: round_size,
    };

    let first: AddressSalt = search_create2_addresses(&params);
    log_new_best(&first);

    let mutex_best = Arc::new(Mutex::new(first));
    let mutex_total_attempts = Arc::new(Mutex::new(1));
    let mutex_total_rounds = Arc::new(Mutex::new(0));

    search(
        &mutex_best,
        &mutex_total_attempts,
        &mutex_total_rounds,
        &params,
        num_rounds,
        num_threads,
    );

    let best = mutex_best.lock().unwrap();

    println!("Best:\n");
    println!(
        "{} zeros {:?} salt 0x{}",
        utils::count_leading_zeroes(best.address),
        best.address,
        hex::encode(utils::bytes32(best.salt_n))
    );
}
