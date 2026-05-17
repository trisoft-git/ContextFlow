use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "contextflow")]
#[command(about = "ContextFlow: Local-first Hyper-Context Autonomous Work Engine", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the daemon
    Start,
    
    /// Show session status
    Status,
    
    /// Summarize current context
    Summarize,
    
    /// Generate execution plan
    Plan,
    
    /// Suggest code fix for last error
    Fix,
    
    /// Manage knowledge items
    Knowledge {
        #[arg(short, long)]
        list: bool,
        
        #[arg(short, long)]
        view: Option<String>,
    },
    
    /// Configuration management
    Config {
        #[arg(long)]
        set: Option<String>,
        
        #[arg(long)]
        get: Option<String>,
    }
}
