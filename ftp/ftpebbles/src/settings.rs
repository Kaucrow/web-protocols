use crate::prelude::*;
use anyhow::Result;
use clap::{ Arg, Command, ArgMatches };

pub struct Settings {
    pub base_dir: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Settings {
    pub fn new(matches: &ArgMatches) -> Result<Self> {
        let username = matches.get_one::<String>("username").cloned();
        let password = matches.get_one::<String>("password").cloned();

        // Ensure username and password are either both set or both unset
        if username.is_some() ^ password.is_some() {
            bail!("Both username and password must be set or neither should be set");
        }

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
                .map_or(Ok(21), |p| p.parse())?,

            username,
            password,
        })
    }
}

pub fn get_command() -> Command {
    Command::new("Five Tiny Pebbles - FTP Server")
        .version("0.1.0")
        .author("Kaucrow")
        .about("Five Tiny Pebbles - FTP Server\n\nA tiny yet efficient FTP server written in Rust.\nNow serving files from the Rubicon!")
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
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .num_args(1)
                .help("Username required for login")
        )
        .arg(
            Arg::new("password")
                .short('P')
                .long("password")
                .num_args(1)
                .help("Password required for login")
        )

}

pub fn get_matches() -> ArgMatches {
    get_command().get_matches()
}