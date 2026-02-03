use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "blockchain-node")]
#[command(about = "A Rust blockchain node")]
pub struct Config {
    /// Port for the REST API
    #[arg(long, env = "API_PORT", default_value_t = 8080)]
    pub api_port: u16,

    /// Port for P2P networking
    #[arg(long, env = "P2P_PORT", default_value_t = 0)]
    pub p2p_port: u16,

    /// Mining difficulty (number of leading zeros)
    #[arg(long, env = "DIFFICULTY", default_value_t = 2)]
    pub difficulty: u32,

    /// Mining reward amount
    #[arg(long, env = "MINING_REWARD", default_value_t = 50)]
    pub mining_reward: u64,
}
