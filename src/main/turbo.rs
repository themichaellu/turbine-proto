extern crate crypto;
extern crate rustc_serialize;
extern crate chrono;
extern crate regex;

use std::thread;
use std::io::{self, Write};
// use std::sync::{Arc, Mutex};
// use network::{server, proto};
use data::{account, state, database, log, profile};
use util::helper;
use postgres::Connection;
use std::io::BufRead;
// use std::sync::mpsc::{self, Sender, Receiver};
use main::commands::*;

pub fn init(){
    println!("\nInitiating Turbine.");
    check_db();
}

pub fn main() {
    init();
    command_loop();
    end();
}

pub fn end(){
    let conn = database::connect_db();
    profile::deactivate(&conn);
    database::close_db(conn);
}

pub fn command_loop(){
    println!("\nInitiating command REPL");
    let mut go: bool = true;
    while go {
        print!(">> ");
        io::stdout().flush().unwrap();
        go = read_command();
    }
}

pub fn check_db(){
    println!("Performing database check...");
    println!("Connecting to database...");
    let conn: Connection = database::connect_db();

    println!("Checking profile database...");
    let missing_tables = database::check_tables(&conn);
    if missing_tables.len() != 0{
        for t in missing_tables{
            println!("Missing table: {:?}. Creating...", t);
            let _ = match &t[..] {
                "account"   => account::create_account_table(&conn),
                "state"     => state::create_state_table(&conn),
                "log"       => log::create_log_table(&conn),
                "profile"   => profile::create_profile_table(&conn),
                _           => {},
            };
        }
    }

    loop {
        //If there are no profiles, create one.
        if profile::num_profile(&conn) == 0{
            println!("No profiles found. Creating one...");
            new_profile();
            break;
        } else {
            println!("\nEnter name of profile to activate: ");
            let name: String = helper::read_in();
            if profile::activate(&name, &conn) {break;}
        }
    }

    //If there are no states, load Genesis.
    if state::num_states(&conn) == 0{
        println!("No saved states.");
        load_genesis(true);
    }
    database::close_db(conn);
}

//Displays commands and flags
pub fn help(){
    let help_text = "Usage: [COMMAND] [FLAGS] [DATA] \nExample: db -drop all \n\n
    Commands \t Options \t Data \t\t Description \n
    profile \t -n \t\t \t\t Create a new profile.\n
    db\t\t -r \t\t [table], all \t Removes a table or the entire database.\n
    genesis\t \t\t \t\t Wipes database and initializes Genesis state \n
    turbo\t \t\t \t\t Connects to network and starts consensus method\n";
    println!("{}", help_text);
}

//Reads and executes a command
pub fn read_command() -> bool{
    let response: String = helper::read_in();
    let split = response.split(" ");
    let raw_vec = split.collect::<Vec<&str>>();
    let mut flags = helper::vec_slice_to_string(&raw_vec);
    let command: String = flags.remove(0);

    let _ = match &command[..]{
        "profile"       => profile_flags(flags),
        "db"            => database_flags(flags),
        "genesis"       => load_genesis(false),
        "turbo"         => turbo(),                 //TODO: Implement running in background
        "help"          => help(),
        "quit"|"exit"   => return false,
        _               => println!("Did not recognize command, please try again."),
    };
    return true;
}

// pub fn end(from_threads: Receiver<String>, connected: Arc<Mutex<Vec<Sender<String>>>>) {
//     //TODO: Close all connections and end threads.
//     proto::close_connections(from_threads, connected);
//
//     //TODO: Deactivate current profile.
//     let conn = database::connect_db();
//     profile::deactivate(&conn);
//
//     //TODO: Remove this.
//     loop{}
// }

#[cfg(test)]
mod test {
  use super::*;
  use main::commands;

  #[test]
  fn test_main() {
      println!("Beginning test...");
      main();
    //   commands::drop_all();
  }
}
