use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::net::TcpStream;
use std::net::SocketAddr;

use crate::proxy_client::proxy::{Credential, NetAddress};


const PROXY_VERSION:    u8 = 5;

// auth const
const NO_AUTH:          u8 = 0;
const PASSWORD_AUTH:    u8 = 2;



#[derive(Debug)]
// socks proxy struct, store the proxy 
// address, port and credential for new connections 
pub struct SocksV5 {
    address: SocketAddr,

    #[allow(dead_code)]
    credential: Option<Credential>,
    greeting_msg: Vec<u8>
}


// public implementation
impl SocksV5 {
    // create a new intance with the specified options
    pub fn new(address: IpAddr, port: u16, credential: Option<Credential>) -> SocksV5 {
        // get the full proxy address
        let proxy_address = SocketAddr::new(address, port);


        // generate the authentication method vector
        let mut auth_methods: Vec<u8> = vec![NO_AUTH];
        if let None = credential {
            auth_methods.push(PASSWORD_AUTH);
        }

        // generate the greeting message
        let mut greeting_msg: Vec<u8> = vec![
            PROXY_VERSION,
            auth_methods.len() as u8
        ];
        
        greeting_msg.extend(&auth_methods);


        // return a proxy client
        SocksV5 {
            address: proxy_address,

            credential,
            greeting_msg
        }
    }


    // ask the proxy to enstablish a TCP connection and return a TCPstream
    pub fn connect_tcp(&self, address: NetAddress, port: u16) -> Result<TcpStream, std::io::Error> {
        let mut proxy_conn = TcpStream::connect(self.address)?;

        // agree on an authentication mode with the server and authenticate
        let auth_mode = self.select_auth_method(&mut proxy_conn)?;
        match auth_mode {
            0 => {},
            2 => {/* TODO: implemenet password log in */},

            _ => return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Unsupported authentication mode"
                ))
        }


        // request a connection
        let conn_status = SocksV5::request_connection(&mut proxy_conn, address, port, 1)?;

        // check the connection status
        match conn_status {
            // if every is ok return the connection to the user
            0 => Ok(proxy_conn),

            1 => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "generic proxy error")),
            3 => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "network unreachable")),
            4 => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "host unreachable")),
            5 => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "connection refused by destination host")),

            // return an error if unknow status is found
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Unknow proxy error"))
        }      
    }

}






// proxy connection handling
impl SocksV5 {
    fn select_auth_method(&self, conn: &mut TcpStream) -> Result<u8, std::io::Error> {
        // send the message
        conn.write(&self.greeting_msg)?;

        // get the server replay
        let mut server_answer: [u8;2] = [0; 2];
        conn.read(&mut server_answer)?;

        // return the auth mode
        Ok(server_answer[1])
    }


    // return a vector structured as follows
    /*  
        [1, 4 bytes for IPv4 address ]
        [3, 1 byte of name length, 1–255 bytes for the domain name ]
        [4, 16 bytes for IPv6 address ] 
    */
    fn generate_address_vec(address: NetAddress) -> Option<Vec<u8>> {
        match address {
            NetAddress::V4(addr_v4) => {
                let mut addr: Vec<u8> = vec![1];
                addr.extend_from_slice(&addr_v4.octets());

                Some(addr)
            },
            NetAddress::V6(addr_v6) => {
                let mut addr: Vec<u8> = vec![4];
                addr.extend_from_slice(&addr_v6.octets());

                Some(addr)
            },

            // if the domain name is longer that 255 byte return None
            NetAddress::Str(addr_str) => {
                if addr_str.len() > 255 { return None; }

                let mut addr: Vec<u8> = vec![3, addr_str.len() as u8];
                addr.extend_from_slice(addr_str.as_bytes());
                
                Some(addr)
            }
        }
    }


    // enstablish a TCP connection througth the proxy
    fn request_connection(conn: &mut TcpStream, address: NetAddress, port: u16, cmd: u8) -> Result<u8, std::io::Error> {
        // get the address vector
        let address_vec = SocksV5::generate_address_vec(address);

        let address_vec = if let Some(addr) = address_vec {
            addr
        } else {
            // return an error if none is found
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Unsupported authentication mode"
            ))
        };
            
        
        // generate command message
        let mut cmd_msg: Vec<u8> = vec![
            PROXY_VERSION,          // server version
            cmd,                    // command byte
            0                       // reverved byte
        ];
        
        // append address
        cmd_msg.extend(address_vec);
        // append port
        cmd_msg.extend(&port.to_be_bytes());

        // send the cmd message
        conn.write(&cmd_msg)?;


        // read the server answer and get status and return address type
        let mut server_answer: [u8; 4] = [0; 4];
        conn.read(&mut server_answer)?;

        let status = server_answer[1];
        let addr_type = server_answer[3];


        // determine addr len and empty buffer
        let buffer_len: usize = match addr_type {
            1 => 6,     // 4 byte ip v4 + 2 port
            4 => 18,    // 16 byte ip v6 + 2 port

            // determine domain name address len
            3 => {
                let mut addr_len: [u8; 1] = [0; 1];
                conn.read(&mut addr_len)?;

                (addr_len[0] as usize) + 2
            },

            // return an error if unknow type is found
            _ => return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Unknow server replay"
                ))
        };

        conn.read(&mut vec![0 ;buffer_len])?;


        // return the status of the connection
        Ok(status)
    }
}