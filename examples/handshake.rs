extern crate bitcoin;

use std::net::{IpAddr, Ipv4Addr, Shutdown, SocketAddr, TcpStream};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, process};
use std::io::Write;

use bitcoin::consensus::encode;
use bitcoin::network::{address, msg};
use bitcoin::network::stream_reader::StreamReader;
use bitcoin::secp256k1;
use bitcoin::secp256k1::rand::Rng;
use bitcoin::{Network, ServiceFlags};

fn main() {
    // This example establishes a connection to a Bitcoin node, sends the intial
    // "version" message, waits for the reply, and finally closes the connection.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("not enough arguments");
        process::exit(1);
    }

    let str_address = &args[1];

    let address: SocketAddr = str_address.parse().unwrap_or_else(|error| {
        eprintln!("Error parsing address: {:?}", error);
        process::exit(1);
    });

    let version_message = build_version_message(address);

    let first_message = msg::RawNetworkMessage {
        magic: Network::Bitcoin.magic(),
        payload: version_message,
    };

    if let Ok(mut stream) = TcpStream::connect(address) {
        // Send the message
        let _ = stream.write_all(encode::serialize(&first_message).as_slice());
        println!("Sent version message");

        // Setup StreamReader
        let read_stream = stream.try_clone().unwrap();
        let mut stream_reader = StreamReader::new(read_stream, None);
        loop {
            // Loop an retrieve new messages
            let reply: msg::RawNetworkMessage = stream_reader.read_next().unwrap();
            match reply.payload {
                msg::NetworkMessage::Version(_) => {
                    println!("Received version message: {:?}", reply.payload);

                    let second_message = msg::RawNetworkMessage {
                        magic: Network::Bitcoin.magic(),
                        payload: msg::NetworkMessage::Verack,
                    };

                    let _ = stream.write_all(encode::serialize(&second_message).as_slice());
                    println!("Sent verack message");
                }
                msg::NetworkMessage::Verack => {
                    println!("Received verack message: {:?}", reply.payload);
                    break;
                }
                _ => {
                    println!("Received unknown message: {:?}", reply.payload);
                    break;
                }
            }
        }
        let _ = stream.shutdown(Shutdown::Both);
    } else {
        eprintln!("Failed to open connection");
    }
}

fn build_version_message(address: SocketAddr) -> msg::NetworkMessage {
    // Building version message, see https://en.bitcoin.it/wiki/Protocol_documentation#version
    let my_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

    // "bitfield of features to be enabled for this connection"
    let services = ServiceFlags::NONE;

    // "standard UNIX timestamp in seconds"
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time error")
        .as_secs();

    // "The network address of the node receiving this message"
    let addr_recv = address::Address::new(&address, ServiceFlags::NONE);

    // "The network address of the node emitting this message"
    let addr_from = address::Address::new(&my_address, ServiceFlags::NONE);

    // "Node random nonce, randomly generated every time a version packet is sent. This nonce is used to detect connections to self."
    let nonce: u64 = secp256k1::rand::thread_rng().gen();

    // "User Agent (0x00 if string is 0 bytes long)"
    let user_agent = String::from("rust-example");

    // "The last block received by the emitting node"
    let start_height: i32 = 0;

    // Construct the message
    msg::NetworkMessage::Version(msg::Version::new(
        services,
        timestamp as i64,
        addr_recv,
        addr_from,
        nonce,
        user_agent,
        start_height,
    ))
}
