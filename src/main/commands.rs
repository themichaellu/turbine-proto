use std::thread;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use network::{server, proto};
use data::{account, state, database, log, profile, tenv};
use util::{helper, genesis};
use postgres::Connection;
use std::io::BufRead;
use std::sync::RwLock;
use std::sync::mpsc::{self, Sender, Receiver};
use main::consensus;
use std::collections::HashMap;
use std::time::Duration;
use vm::env;

//====================================================================
// COMMAND FUNCTIONS
// Contains functions called by the command loop.
//====================================================================

// TODO: Error after drop all and exiting program.
// Drops all database tables
pub fn drop_all(){
        let conn = database::connect_db();
        account::drop_account_table(&conn);
        state::drop_state_table(&conn);
        log::drop_log_table(&conn);
        profile::drop_profile_table(&conn);
        database::close_db(conn);
}

// TODO: Error when drop all and continuing. Need to break.
// Drops database and loads Genesis state.
pub fn load_genesis(init: bool){

    // If init is true, it means its a fresh install so there isn't a need for a prompt
    if !init {
        println!("Loading Genesis state erases the database. Continue? (y/n)");
        let yn = helper::read_yn();
        if !yn {
            return;
        }
        drop_all();
    }

    let genesis_state = genesis::get_genesis();
    let conn = database::connect_db();
    state::save_state(&genesis_state, &conn);
    database::close_db(conn);
    println!("Genesis state loaded.");
}

//Execute db command with flags
pub fn database_flags(flags: Vec<String>){
    let flag = &flags[0];
    match &flag[..]{
        "-r" => {
            let conn = database::connect_db();
            let target = &flags[1];
            match &target[..]{
                "all" => {
                    println!("Are you sure you want to drop everything? (y/n)");
                    let yn: bool = helper::read_yn();
                    if yn {
                        drop_all();
                    }
                },
                "account"   => account::drop_account_table(&conn),
                "profile"   => {
                    profile::deactivate(&conn);
                    profile::drop_profile_table(&conn);
                },
                "log"       => log::drop_log_table(&conn),
                "state"     => state::drop_state_table(&conn),
                _           => println!("Unrecognized flag target for [db -drop]"),
            };
            database::close_db(conn);
        },
        _ => println!("Unrecognized flags for command [db]"),
    }
}

//Execute profile command with flags
pub fn profile_flags(flags: Vec<String>){
    if flags.len() == 0 {
        println!("Profile command requires flags.");
    } else {
        let flag = &flags[0];
        match &flag[..]{
            "-n"    => new_profile(),
            _       => println!("Unrecognized flags for command [profile]"),
        }
    }
}

//Creates a new profile.
pub fn new_profile(){
    let conn = database::connect_db();
    println!("\nEnter the name of new profile:");
    let n = helper::read_in();

    //TODO: Change from static IP to one that the user enters
    println!("Enter the IP address and port (ex. 127.0.0.1:8888):");
    let ip = helper::read_in();
    // let ip = "127.0.0.1:8888";

    profile::new_profile(&n, &ip, &conn);

    //TODO: Profile can fail.
    println!("Profile created.");
    database::close_db(conn);
}

// Main method
// Connects to network and starts consensus loop
pub fn turbo(){
    println!("\n\nPerforming network check...");
    let conn = database::connect_db();
    let p = profile::get_active(&conn).unwrap();
    database::close_db(conn);
    let trusted: Vec<String> = helper::slice_to_vec(&p.trusted);
    let local_ip: String = p.ip;

    println!("Starting local server...");

    // Initializing Arcs
    // Local Status. String<Status>
    let main_stat: Arc<RwLock<(String, String)>> = Arc::new(RwLock::new((String::new(), String::new())));
    // Connected Nodes and their current status. HashMap<Address, (State, Nonce)>
    let thread_stat: Arc<RwLock<HashMap<String, tenv::tenv>>> = Arc::new(RwLock::new(HashMap::new()));
    // Current Accounts. HashMap<Address, Account>
    // let curr_accs: Arc<RwLock<HashMap<String, account::account>>> = Arc::new(RwLock::new(HashMap::new()));
    // Current Logs. HashMap<Hash, Log>
    let curr_logs: Arc<RwLock<HashMap<String, log::log>>> = Arc::new(RwLock::new(HashMap::new()));
    // Cloning to move into server
    let m_stat = main_stat.clone();
    let t_stat = thread_stat.clone();
    // let c_accs = curr_accs.clone();
    let c_logs = curr_logs.clone();
    //Starting Server
    let _ = thread::spawn(move ||
        server::listen(local_ip, m_stat, t_stat, c_logs)
    );

    //Connecting to trusted accounts for active profile.
    println!("\nThere are {:?} trusted accounts on this profile.", trusted.len());
    thread::sleep(Duration::from_millis(500)); // Allow server to bind

    // Connecting to peers
    for ip in trusted{
        server::connect(&ip, main_stat.clone(), thread_stat.clone(), curr_logs.clone());
    }
    let tenv_arc = thread_stat.clone();

    //TODO: Reimplement
    // println!("Connected to {:?} peers.", conn_len);
    //
    // for add in tenv_arc.read().unwrap().keys(){
    //     println!("Connected to node: {:?}", add);
    // }

    //Starts consensus loop
    println!("Starting consensus.");
    consensus::consensus_loop(main_stat, tenv_arc, curr_logs);
}
