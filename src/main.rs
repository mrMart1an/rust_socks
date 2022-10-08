mod proxy_client;

use proxy_client::proxy::NetAddress;
use proxy_client::socks_v5::SocksV5;
use std::{net::{IpAddr, Ipv4Addr}, io::{Write, Read}};

fn main() {
    let proxy = SocksV5::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9050, None);

    //let address = NetAddress::Str("en.wikipedia.org".to_string());
    let address = NetAddress::Str("www.duckduck.com".to_string());
    let mut conn = proxy.connect_tcp(address, 80).unwrap();

    //let mut conn = std::net::TcpStream::connect("31.11.33.60:80").unwrap();
    
    let header = "GET / HTTP/1.1\r\n\r\n".as_bytes();

    conn.write(header).unwrap();

    let mut server_answer: Vec<u8> = vec![0; 4096];
    conn.read(&mut server_answer).unwrap();

    

    println!("{}", String::from_utf8_lossy(&server_answer));
    


}
