use anyhow::Result;
use clap::{ Arg, Command, ArgMatches };

pub struct Settings {
    pub base_dir: String,
    pub host: String,
    pub port: u16,
}

impl Settings {
    pub fn new(matches: &ArgMatches) -> Result<Self> {
        Ok(Self {
            base_dir: matches
                .get_one::<String>("directory")
                .cloned()
                .unwrap_or("C:/Users".to_string()),

            host: matches
                .get_one::<String>("host")
                .cloned()
                .unwrap_or("127.0.0.1".to_string()),

            port: matches
                .get_one::<String>("port")
                .map_or(Ok(21), |p| p.parse())?
        })
    }
}

pub fn get_command() -> Command {
    Command::new("Rust FTP Server")
        .version("0.1.0")
        .author("Kaucrow")
        .about(r"A smol Rust FTP server \( ᐛ )/")
        .arg(
            Arg::new("directory")
                .short('d')
                .long("dir")
                .num_args(1)
                .help("Directory to host files from"),
        )
        .arg(
            Arg::new("host")
                .short('H')
                .long("host")
                .num_args(1)
                .help("Server hostname"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .num_args(1)
                .help("Server port"),
        )

}

pub fn get_matches() -> ArgMatches {
    get_command().get_matches()
}