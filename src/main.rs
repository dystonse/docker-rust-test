mod importer;
mod analyser;

use std::error::Error;
#[macro_use]
extern crate lazy_static;

use std::fs;
use std::fs::File;
use std::io::prelude::*;

use clap::{App, Arg, ArgMatches};
use mysql::*;
use retry::delay::Fibonacci;
use retry::retry;
use serde::Serialize;

use importer::Importer;
use analyser::Analyser;

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum EventType {
    Arrival,
    Departure,
}

// This is handy, because mysql defines its own Result type and we don't
// want to repeat std::result::Result
type FnResult<R> = std::result::Result<R, Box<dyn Error>>;

pub struct Main {
    verbose: bool,
    pool: Pool,
    args: ArgMatches,
    source: String,
}

fn main() -> FnResult<()> {
    let mut instance = Main::new()?;
    instance.run()?;
    Ok(())
}

fn parse_args() -> ArgMatches {
    let matches = App::new("dystonse-gtfs-data")
        .subcommand(Importer::get_subcommand())
        .subcommand(Analyser::get_subcommand())
        .arg(Arg::new("verbose")
            .short('v')
            .long("verbose")
            .about("Output status messages during run.")
        ).arg(Arg::new("password")
            .short('p')
            .long("password")
            .env("DB_PASSWORD")
            .takes_value(true)
            .about("Password used to connect to the database.")
            .required_unless("help")
        ).arg(Arg::new("user")
            .short('u')
            .long("user")
            .env("DB_USER")
            .takes_value(true)
            .about("User on the database.")
            .default_value("dystonse")
        ).arg(Arg::new("host")
            .long("host")
            .env("DB_HOST")
            .takes_value(true)
            .about("Host on which the database can be connected.")
            .default_value("localhost")   
        ).arg(Arg::new("port")
            .long("port")
            .env("DB_PORT")
            .takes_value(true)
            .about("Port on which the database can be connected.")
            .default_value("3306")
        ).arg(Arg::new("database")
            .short('d')
            .long("database")
            .env("DB_DATABASE")
            .takes_value(true)
            .about("Database name which will be selected.")
            .default_value("dystonse")
        ).arg(Arg::new("source")
            .short('s')
            .long("source")
            .env("GTFS_DATA_SOURCE_ID")
            .takes_value(true)
            .about("Source identifier for the data sets. Used to distinguish data sets with non-unique ids.")
            .required_unless("help")
        )
        .get_matches();
    return matches;
}

impl Main {
    /// Constructs a new instance of Main, with parsed arguments and a ready-to-use pool of database connections.
    fn new() -> FnResult<Main> {
        let args = parse_args();
        let verbose = args.is_present("verbose");
        let source = String::from(args.value_of("source").unwrap()); // already validated by clap

        if verbose {
            println!("Connecting to database…");
        }
        let pool = retry(Fibonacci::from_millis(1000), || {
            Main::open_db(&args, verbose)
        })
        .expect("DB connections should succeed eventually.");
        Ok(Main {
            args,
            verbose,
            pool,
            source,
        })
    }

    /// Runs the actions that are selected via the command line args
    fn run(&mut self) -> FnResult<()> {
        match self.args.clone().subcommand() {
            ("import", Some(sub_args)) => {
                let mut importer = Importer::new(&self, sub_args);
                importer.run()
            }
            ("analyse", Some(sub_args)) => {
                let mut analyser = Analyser::new(&self, sub_args);
                analyser.run()
            }
            _ => panic!("Invalid arguments."),
        }
    }

    /// Opens a connection to a database and returns the resulting connection pool.
    /// Takes configuration values from DB_PASSWORD, DB_USER, DB_HOST, DB_PORT and DB_DATABASE
    /// environment variables. For all values except DB_PASSWORD a default is provided.
    fn open_db(args: &ArgMatches, verbose: bool) -> FnResult<Pool> {
        if verbose {
            println!("Trying to connect to the database.");
        }
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            args.value_of("user").unwrap(), // already validated by clap
            args.value_of("password").unwrap(), // already validated by clap
            args.value_of("host").unwrap(), // already validated by clap
            args.value_of("port").unwrap(), // already validated by clap
            args.value_of("database").unwrap()  // already validated by clap
        );
        let pool = Pool::new(url)?;
        Ok(pool)
    }

}

pub enum SerdeFormat {
    Json,
    MessagePack
}

pub fn save_to_file(object: &impl Serialize, dir_name: &str, file_name: &str, format: SerdeFormat) -> FnResult<()> {
    let serialized_bin = match format {
        SerdeFormat::MessagePack => rmp_serde::to_vec(object).unwrap(),
        SerdeFormat::Json => serde_json::to_vec(object).unwrap(),
    };
    fs::create_dir_all(&dir_name)?;    
    let file_path = format!("{}/{}", dir_name, file_name);
    let mut file = match File::create(&file_path) {
        Err(why) => panic!("couldn't create file: {}", why),
        Ok(file) => file,
    };
    match file.write_all(&serialized_bin) {
        Err(why) => panic!("couldn't write: {}", why),
        Ok(_) => println!("successfully wrote."),
    }

    Ok(())
}