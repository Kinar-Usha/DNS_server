use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "DNS Server")]
#[command(about = "A recursive DNS server with configurable cache and threading", long_about = None)]
pub struct Config {
    /// Port to bind the DNS server to
    #[arg(short, long, default_value = "2053")]
    pub port: u16,

    /// Maximum number of LRU cache entries
    #[arg(short, long, default_value = "1000")]
    pub cache_size: usize,

    /// Number of Tokio worker threads (0 = auto-detect based on CPU cores)
    #[arg(short, long, default_value = "0")]
    pub threads: usize,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "debug")]
    pub log_level: String,

    /// IP address to bind to
    #[arg(long, default_value = "0.0.0.0")]
    pub bind: String,
}

impl Config {
    /// Parse command-line arguments and validate configuration
    pub fn parse_and_validate() -> Self {
        let mut config = Self::parse();
        
        // Validate cache_size
        if config.cache_size < 10 {
            eprintln!("⚠ Cache size must be at least 10, setting to 10");
            config.cache_size = 10;
        }
        if config.cache_size > 100000 {
            eprintln!("⚠ Cache size exceeds maximum of 100000, setting to 100000");
            config.cache_size = 100000;
        }

        // Validate log_level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&config.log_level.to_lowercase().as_str()) {
            eprintln!("⚠ Invalid log level '{}', using 'debug'", config.log_level);
            config.log_level = "debug".to_string();
        } else {
            config.log_level = config.log_level.to_lowercase();
        }

        config
    }

    /// Get the full socket address (bind address + port)
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.bind, self.port)
    }

    /// Print configuration on startup
    pub fn print_startup_info(&self) {
        eprintln!("╔════════════════════════════════════════╗");
        eprintln!("║      DNS Server Configuration         ║");
        eprintln!("╠════════════════════════════════════════╣");
        eprintln!("║ Listen Address: {:<22} ║", self.socket_addr());
        eprintln!("║ Cache Size:     {:<22} ║", self.cache_size);
        eprintln!("║ Worker Threads: {:<22} ║", if self.threads == 0 { "auto".to_string() } else { self.threads.to_string() });
        eprintln!("║ Log Level:      {:<22} ║", self.log_level);
        eprintln!("╚════════════════════════════════════════╝");
    }
}
