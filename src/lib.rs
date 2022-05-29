use std::sync::{Arc, RwLock};

#[derive(Default, Debug)]
pub struct Config {
    pub dynamic_fees: bool,
    pub dynamic_fee_min: i64,
    pub dynamic_fee_max: i64,
}

impl Config {
    pub fn default() -> Config {
        Config { 
            dynamic_fees: false, dynamic_fee_min: 0, dynamic_fee_max: 1000
        }
    }

    pub fn current() -> Arc<Config> {
        CURRENT_CONFIG.with(|c| c.read().unwrap().clone())
    }
    pub fn make_current(self) {
        CURRENT_CONFIG.with(|c| *c.write().unwrap() = Arc::new(self))
    }
}

thread_local! {
    static CURRENT_CONFIG: RwLock<Arc<Config>> = RwLock::new(Default::default());
}