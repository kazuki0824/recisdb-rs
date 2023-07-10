use env_logger::Env;

pub(crate) fn initialize_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}
