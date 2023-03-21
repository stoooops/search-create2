use std::sync::{Arc, Mutex};

use ethers_core::{
    types::{Address, Bytes, U256},
    utils::get_create2_address_from_hash,
};

use num_format::{Locale, ToFormattedString};
use rayon::{prelude::*, ThreadPool, ThreadPoolBuilder};

use crate::utils::{bytes32, count_leading_zeroes, fmt_dms};

#[derive(Copy, Clone)]
pub struct AddressSalt {
    pub address: Address,
    // leading_zeros: u8,
    pub salt_n: U256,
}

#[derive(Copy, Clone)]
pub struct SearchParams {
    pub deployer: Address,
    pub initial_salt_n: U256,
    pub init_code_hash: U256,
    pub round_size: u128,
    pub num_rounds: u128,
}

pub struct Searcher {
    best: Arc<Mutex<Option<AddressSalt>>>,
    total_attempts: Arc<Mutex<u128>>,
    total_rounds: Arc<Mutex<u128>>,
    thread_pool: ThreadPool,
}

impl Searcher {
    pub fn new(num_threads: usize) -> Self {
        let best = Arc::new(Mutex::new(None));
        let total_attempts = Arc::new(Mutex::new(0));
        let total_rounds = Arc::new(Mutex::new(0));

        // Create a custom thread pool with the specified number of threads
        let thread_pool = ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("Failed to create thread pool");

        Self {
            best,
            total_attempts,
            total_rounds,
            thread_pool,
        }
    }

    pub fn search(&self, params: SearchParams) -> AddressSalt {
        let start_time = std::time::Instant::now();

        self.thread_pool.install(|| {
            (0..params.num_rounds).into_par_iter().for_each(|round| {
                self.search_round(&params, round, start_time);
            });
        });

        let the_best = self.best.lock().unwrap();
        return the_best.unwrap();
    }

    fn search_round(
        &self,
        initial_params: &SearchParams,
        round: u128,
        start_time: std::time::Instant,
    ) -> AddressSalt {
        let SearchParams {
            deployer,
            initial_salt_n,
            init_code_hash,
            round_size,
            num_rounds,
        } = initial_params;

        let round_offset = U256::from(*round_size) * U256::from(round);
        let round_salt_n = initial_salt_n + round_offset;
        // let round_salt = bytes32(round_salt_n);
        let params = SearchParams {
            deployer: *deployer,
            initial_salt_n: round_salt_n,
            init_code_hash: init_code_hash.clone(),
            round_size: *round_size,
            num_rounds: *num_rounds,
        };

        let round_best = Self::search_create2_addresses(&params);
        // acquire best mutex and check if there are more leading zeros
        let mut best_mutex = self.best.lock().unwrap();
        let mut total_rounds = self.total_rounds.lock().unwrap();
        *total_rounds += 1;
        let mut total_attempts = self.total_attempts.lock().unwrap();
        *total_attempts += round_size;
        // this will be unlocked when the lock goes out of scope which is when the function returns

        // update best
        if best_mutex.is_none() || round_best.address < best_mutex.unwrap().address {
            *best_mutex = Some(round_best);
            Self::log_new_best(&best_mutex.unwrap());
        } else if *total_rounds % 100 == 0 {
            // periodically log the best
            Self::log_best(&best_mutex.unwrap());
        }

        Self::log_attempts(
            *total_rounds,
            *total_attempts,
            start_time,
            count_leading_zeroes(best_mutex.unwrap().address),
        );
        return round_best;
    }

    /// Search for the CREATE2 address with lowest value (i.e. most leading zeros)
    ///
    /// # Arguments
    /// * `params` - The search parameters
    ///
    /// # Returns
    /// * The address with the lowest value found in the search
    fn search_create2_addresses(params: &SearchParams) -> AddressSalt {
        let SearchParams {
            deployer,
            initial_salt_n,
            init_code_hash,
            round_size,
            num_rounds: _,
        } = params;
        let mut salt_n = *initial_salt_n;
        let mut salt = bytes32(salt_n);

        let init_code_hash_bytes: Bytes = bytes32(*init_code_hash);

        let mut address: Address =
            get_create2_address_from_hash(*deployer, &salt, &init_code_hash_bytes);

        let mut best: AddressSalt = AddressSalt {
            address: address,
            // leading_zeros: address.leading_zeros,
            salt_n,
        };

        // already checked the first address
        for _i in 0..*round_size - 1 {
            salt_n += U256::from(1);
            salt = bytes32(salt_n);
            address = get_create2_address_from_hash(*deployer, &salt, &init_code_hash_bytes);
            // check if we have a new best
            if address < best.address {
                best = AddressSalt { address, salt_n };
            }
        }
        return best;
    }

    /// Log the round/attempts/etc.
    ///
    /// # Arguments
    /// * `round` - The round number
    /// * `attempt` - The number of attempts in this round
    /// * `now` - The time at the start of the round
    /// * `best_zeros` - The number of leading zeros in the best address found so far
    ///
    /// # Returns
    /// * None
    ///
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
            Self::fmt_countdown(best_zeros + 1, rate, elapsed_ms)
        );
    }

    /// Log the best address found so far
    ///
    /// # Arguments
    /// * `best` - The best address found so far
    ///
    /// # Returns
    /// * None
    fn log_best(best: &AddressSalt) {
        let msg = format!(
            "{} zeros {:?} salt 0x{}",
            count_leading_zeroes(best.address),
            best.address,
            hex::encode(bytes32(best.salt_n))
        );
        // print to terminal in cyan ANSI color
        println!("\x1b[36m{}\x1b[0m", msg);
    }

    /// Log a newly found best address
    ///
    /// # Arguments
    /// * `best` - The best address found so far
    ///
    /// # Returns
    /// * None
    fn log_new_best(best: &AddressSalt) {
        let msg = format!(
            "{} zeros {:?} salt 0x{}",
            count_leading_zeroes(best.address),
            best.address,
            hex::encode(bytes32(best.salt_n))
        );
        // print to terminal in green ANSI color
        println!("\x1b[32m{}\x1b[0m", msg);
    }

    /// Format the countdown to the next leading zero
    /// e.g. (5 0s T-1d 2h 3m 4s)
    ///
    /// # Arguments
    /// * `zeros` - The number of leading zeros
    /// * `rate` - The rate of attempts per second
    /// * `elapsed_ms` - The number of milliseconds elapsed
    ///
    /// # Returns
    /// * A string in the format "(X 0s T-YdZhSmSs)" where X is the number of leading zeros,
    /// Y is the number of days, Z is the number of hours, S is the number of minutes, and S is
    /// the number of seconds.
    fn fmt_countdown(zeros: u8, rate: f64, elapsed_ms: u128) -> String {
        let expected_attempts: u128 = 16_u128.pow(zeros as u32);
        let expected_attempts_secs_at_current_rate = expected_attempts as f64 / rate;
        // this is a statistical fallacy, but humans want to see progress
        let expected_remaining_time_at_rate =
            (expected_attempts_secs_at_current_rate - (elapsed_ms as f64 / 1000.0)) as u128;
        return format!(
            " ({} 0s T-{})",
            zeros,
            fmt_dms(expected_remaining_time_at_rate)
        );
    }
}
