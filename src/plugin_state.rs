pub mod plugin_state {

    #[derive(Clone, Debug)]
    pub struct PluginState {
        pub initialized: bool,
        pub config: Config,
    }

    impl PluginState {
        pub fn initialize(&mut self) -> &mut PluginState {
            self.initialized = true;
            self
        }

        pub fn enable_dynamic_fees(&mut self) -> &mut PluginState {
            self.config.dynamic_fees = true;
            self
        }
    }

    #[derive(Clone, Debug)]
    pub struct Config {
        pub dynamic_fees: bool
    }

    impl Config {
        pub fn new() -> Config {
            Config {
                dynamic_fees: false
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::PluginState;
    use crate::Config;

    #[test]
    fn it_can_be_initialized() {
        let mut ps = PluginState {
            initialized: false,
            config: Config::new()
        };

        ps.initialize();

        assert!(ps.initialized);
    }

    #[test]
    fn it_can_enable_dynamic_fees() {
        let mut ps = PluginState {
            initialized: true,
            config: Config {
                dynamic_fees: false
            }
        };

        ps.enable_dynamic_fees();

        assert!(ps.config.dynamic_fees);
    }
}