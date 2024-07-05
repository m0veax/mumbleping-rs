use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use bincode::{Decode, Encode};
use std::thread;

#[derive(Debug,Encode,Decode)]
#[repr(C)]
pub struct Pong {
    server_version: u32,
    ident: u64,
    connected_users: u32,
    max_users: u32,
    bandwidth: u32
}


/*
Width 	    Data type 	Value 	Comment
4 bytes 	int 	    0 	    Denotes the request type
8 bytes 	long long 	ident 	Used to identify the reponse.
*/
#[derive(Debug,Encode,Decode)]
#[repr(C)]
pub struct Ping {
    ping: u32,
    identifier: u64
}


pub fn get_mumble_data(mumble_remote: &str) -> Pong{

    // create "client" udp socket on a OS choosen port
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind to address");

    let identifier: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let ping = Ping {
        ping: 0,
        identifier
    };

    let mut slice = [0u8; 12];

    // struct to binary :)
    let length = bincode::encode_into_slice(
        ping,
        &mut slice,
        bincode::config::standard()
    ).unwrap();

    let slice = &slice[..length];

    // time to send ping
    socket.send_to(&slice, &mumble_remote).expect("couldn't send data");

    // receive pong
    //get request from socket
    let mut buf = [0; 24];
    socket.recv_from(&mut buf).expect("Failed to receive data");

    /*
        server_version: u32,
    last_update: u64,
    connected_users: u32,
    max_users: u32,
    bandwidth: u32
     */

    let (server_version, ident, connected_users, max_users, bandwidth): (u32, u64, u32, u32, u32) = bincode::decode_from_slice(&buf, bincode::config::standard().with_big_endian()).unwrap().0;

    let pong = Pong {
        server_version,
        ident,
        connected_users,
        max_users,
        bandwidth
    };

    return pong;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_server() {

        let socket = UdpSocket::bind("127.0.0.1:64738").expect("Failed to bind to address");

        //get request from socket
        let mut buf = [0; 12];
        let (size, source) = socket.recv_from(&mut buf).expect("Failed to receive data");

        let (ping, identifier): (u32, u64) = bincode::decode_from_slice(&buf, bincode::config::standard().with_big_endian()).unwrap().0;

        /*
        The response will then contain the following data:
        Width 	    Data type 	Value 	Comment
        4 bytes 	int 	Version 	e.g., \x0\x1\x2\x3 for 1.2.3. Can be interpreted as one single int or four signed chars.
        8 bytes 	long long 	ident 	the ident value sent with the request
        4 bytes 	int 	Currently connected users count
        4 bytes 	int 	Maximum users (slot count)
        4 bytes 	int 	Allowed bandwidth
         */

        let pong = Pong {
            server_version: 123,
            ident: identifier,
            max_users: 12,
            connected_users: 2,
            bandwidth: 312
        };

        let mut slice = [0u8; 24];

        let length = bincode::encode_into_slice(
            &pong,
            &mut slice,
            bincode::config::standard().with_big_endian()
        ).unwrap();

        let slice = &slice[..length];

        // send Pong
        socket.send_to(&slice, source).expect("couldn't send data");
    }

    #[test]
    fn it_works() {
        let mumble_remote = "127.0.0.1:64738";

        // create "server" port for test
        thread::spawn(|| {
            test_server();
        });

        let result = get_mumble_data(mumble_remote);

        let max_users: u32 = 12;

        assert_eq!(&result.max_users, &max_users);
    }
}
