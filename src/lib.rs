use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};

use bincode::{Decode, Encode};
use thiserror::Error;

/*
Width 	    Data type 	Value 	Comment
4 bytes 	int 	Version 	e.g., \x0\x1\x2\x3 for 1.2.3. Can be interpreted as one single int or four signed chars.
8 bytes 	long long 	ident 	the ident value sent with the request
4 bytes 	int 	Currently connected users count
4 bytes 	int 	Maximum users (slot count)
4 bytes 	int 	Allowed bandwidth
*/
#[derive(Debug, Encode, Decode)]
#[repr(C)]
pub struct Pong {
    server_version: u32,
    ident: u64,
    connected_users: u32,
    max_users: u32,
    bandwidth: u32,
}

/*
Width 	    Data type 	Value 	Comment
4 bytes 	int 	    0 	    Denotes the request type
8 bytes 	long long 	ident 	Used to identify the reponse.
*/
#[derive(Debug, Encode, Decode)]
#[repr(C)]
pub struct Ping {
    ping: u32,
    identifier: u64,
}

#[derive(Debug, Error)]
pub enum MumbleError {
    #[error("could not receive data: {0}")]
    Receive(std::io::Error),
    #[error("could not send data: {0}")]
    Send(std::io::Error),
    #[error("could not encode data: {0}")]
    Encode(bincode::error::EncodeError),
    #[error("could not decode data: {0}")]
    Decode(bincode::error::DecodeError),
}

const ENCODING_CFG: bincode::config::Configuration<
    bincode::config::BigEndian,
    bincode::config::Fixint,
> = bincode::config::standard()
    .with_big_endian()
    .with_fixed_int_encoding();

pub fn get_mumble_data(mumble_remote: &str) -> Result<Pong, MumbleError> {
    // create "client" udp socket on a OS choosen port
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind to address");

    let identifier: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let ping = Ping {
        ping: 0,
        identifier,
    };

    let mut slice = [0u8; 12];

    // struct to binary :)
    let length =
        bincode::encode_into_slice(ping, &mut slice, ENCODING_CFG).map_err(MumbleError::Encode)?;
    let slice = &slice[..length];

    // time to send ping
    socket
        .send_to(slice, mumble_remote)
        .map_err(MumbleError::Send)?;

    // receive pong
    //get request from socket
    let mut buf = [0; 24];
    socket.recv_from(&mut buf).map_err(MumbleError::Receive)?;

    let (server_version, ident, connected_users, max_users, bandwidth): (u32, u64, u32, u32, u32) =
        bincode::decode_from_slice(&buf, ENCODING_CFG)
            .map_err(MumbleError::Decode)?
            .0;

    Ok(Pong {
        server_version,
        ident,
        connected_users,
        max_users,
        bandwidth,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_server() {
        let socket = UdpSocket::bind("0.0.0.0:64738").expect("Failed to bind to address");

        //get request from socket
        let mut buf = [0; 12];
        let (_size, source) = socket.recv_from(&mut buf).expect("Failed to receive data");

        let (_ping, identifier): (u32, u64) =
            bincode::decode_from_slice(&buf, ENCODING_CFG).unwrap().0;

        let pong = Pong {
            server_version: 123,
            ident: identifier,
            max_users: 12,
            connected_users: 2,
            bandwidth: 312,
        };

        let mut slice = [0u8; 24];

        let length = bincode::encode_into_slice(&pong, &mut slice, ENCODING_CFG).unwrap();

        let slice = &slice[..length];

        // send Pong
        socket.send_to(slice, source).expect("couldn't send data");
    }

    #[test]
    fn it_works() {
        let mumble_remote = "127.0.0.1:64738";

        // create "server" port for test
        std::thread::spawn(|| {
            test_server();
        });

        let result = get_mumble_data(mumble_remote).unwrap();

        let max_users: u32 = 12;

        assert_eq!(&result.max_users, &max_users);
    }
}
