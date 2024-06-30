
use std::net::UdpSocket;


struct Pong {
    server_version: String,
    last_update: u64,
    connected_users: u32,
    max_users: u32,
    bandwidth: u32
}


pub fn get_mumble_data(mumble_remote: &str) -> i32{

    // create "client" udp socket on a OS choosen port
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind to address");

    let message = "Hello Server";
    socket.send_to(message.as_bytes(), &mumble_remote).expect("couldn't send data");

    let result = 4;


    return result;
}


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // create "server" port for test
        let socket = UdpSocket::bind("127.0.0.1:64738").expect("Failed to bind to address");

        let mumble_remote = "127.0.0.1:64738";

        let result = get_mumble_data(mumble_remote);

        let mut buf = [0; 1024];
        let (size, source) = socket.recv_from(&mut buf).expect("Failed to receive data");
        let request = String::from_utf8_lossy(&buf[..size]);
        println!("Received request: {} from {}", request, source);


        assert_eq!(result, 4);
    }
}
