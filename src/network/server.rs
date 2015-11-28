// Generic TCP Server Implementation
extern crate postgres;


use std::net::{TcpStream, TcpListener, SocketAddrV4, Ipv4Addr};
use std::{thread, str};
use std::time::Duration;

use std::io::Read;
use std::io::Write;

use data::account::account;
use data::database;
use network::proto;
use postgres::{Connection, SslMode};
use util::{helper, krypto};
use std::sync::mpsc::{self, Sender, Receiver};



// pub fn listen(listen_addr: SocketAddrV4) {
pub fn listen(address: String, to_thread: Sender<String>){
	let add: &str  = &address;
	let listener = TcpListener::bind(add).unwrap();
	println!("\nListening started on {}", address);
	let _ = to_thread.send("bound".to_string());

    for stream in listener.incoming() {
		let thread_tx = to_thread.clone();
    	match stream {
    		Err(e) => { println!("Error on listening: {}", e) }
    		Ok(stream) => {
    			thread::spawn(move || {
					//This only works on nightly it seems. Need to make the switch.
					// stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
    				handle(stream, thread_tx);
    			});
    		}
    	}
	}
}

//Connect to IP address.
pub fn connect(address: &str, to_thread: Sender<String>) -> bool{
	let stream_attempt = TcpStream::connect(address);
	match stream_attempt {
		Ok(stream) => {
			thread::spawn(move||{
				handle(stream, to_thread);
			});
			return true;
		},
		Err(_) => false,
	}
}

fn handle(mut stream: TcpStream, to_thread: Sender<String>) {
	println!("Connected. Passed to handler");
	let mut proto_buf;
	// proto::handshake(&mut stream, &conn);
	let _ = stream.write(&[2, 0]);
	loop {
		proto_buf = [0; 2];
		let _ = match stream.read(&mut proto_buf) {
			Err(e) 	=> panic!("Error on read: {}", e),
			Ok(_) 	=> match_proto(&proto_buf[..], &mut stream),
		};
	}
	println!("Finished reading from stream.");
	drop(stream);
}

pub fn ping(stream: &mut TcpStream)-> bool{
	let mut inc = [0;2];
	let _ = stream.write(&[0, 0]);
	let b: bool = match stream.read(&mut inc){
		Err(_) 		=> false,
		Ok(_) 		=> {
			if inc[0] == 1 {
				return true;
			} else {
				return false;
			}
		},
	};
	return b;
}

fn match_proto(incoming: &[u8], mut stream: &mut TcpStream){
	match incoming[0]{
		0			=> {
							println!("Incoming message >> Ping");
							//Sending Pong
							//TODO: Should only send pong if not blacklisted.
							let _ = stream.write(&[1,0]);
						},
		1			=> {

						},
		2			=> {
							println!("Incoming message >> Requesting Handshake");
							proto::send_handshake(stream);
						},
		3			=> {
							println!("Incoming message >> Sending Handshake");
							println!("Their handshake: {:?}", read_stream(stream, incoming[1]));
						},
		4			=> {
							println!("Incoming message >> Requesting Logs");
							let raw_hash = read_stream(stream, incoming[1]);
							let hash = String::from_utf8(raw_hash).unwrap();
							proto::send_log(stream, hash);
						},
		5			=> {
							println!("Incoming message >> Sending Logs");
							println!("Their logs: {:?}", read_stream(stream, incoming[1]));
						},
		6			=> {
							println!("Incoming message >> Requesting Account");
							let raw_address = read_stream(stream, incoming[1]);
							let address = String::from_utf8(raw_address).unwrap();
							proto::send_account(stream, address);
						},
		7			=> {
							println!("Incoming message >> Sending Account");
							println!("Their account: {:?}", read_stream(stream, incoming[1]));
						},
		8			=> {
							println!("Incoming message >> Requesting State");
							let raw_hash = read_stream(stream, incoming[1]);
							let hash = String::from_utf8(raw_hash).unwrap();
							proto::send_state(stream, hash);
						},
		9			=> {
							println!("Incoming message >> Sending State");
							println!("Their state: {:?}", read_stream(stream, incoming[1]));
						},
		16			=> {
							println!("Incoming message >> Update State");
						},
		17 			=> println!("what is this."),
		_			=> println!("matches nothing."),
	}
}

fn read_stream(stream: &mut TcpStream, length: u8) -> Vec<u8>{
	let mut data_buf = vec![0; length as usize];
	let _ = stream.read(&mut data_buf[..]);
	return data_buf;
}

// #[cfg(test)]
// mod test {
//   use std::net;
//   use std::thread;
//   use super::*;
//
//   #[test]
//   fn test_tcp() {
//     println!("TCP server test");
//     let ip = net::Ipv4Addr::new(127, 0, 0, 1);
//     let listen_addr = net::SocketAddrV4::new(ip, 8888);
// 	let send_addr = net::SocketAddrV4::new(ip, 8888);
//     let listener = listen(net::SocketAddr::V4(listen_addr));
//   }
// }
