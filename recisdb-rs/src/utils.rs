use env_logger::Env;

pub(crate) enum StreamExitType {
    Success(u64),
    Timeout,
    Error(std::io::Error),
    UnexpectedEofInTuner,
}

pub(crate) fn initialize_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}
