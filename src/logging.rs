use env_logger;

pub fn init() {
    env_logger::init();
    log::info!("Logging initialized");
}
