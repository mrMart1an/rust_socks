#[derive(Debug)]
#[allow(dead_code)]
// store the credential for the proxy access
pub struct Credential {
    user: String,
    password: String
}

#[allow(dead_code)]
impl Credential {
    // generate a credential instace
    pub fn new(user: String, password: String) -> Credential {
        Credential {user, password}
    }
}


#[allow(dead_code)]
pub enum NetAddress {
    V4(std::net::Ipv4Addr),
    V6(std::net::Ipv6Addr),
    Str(String)
}


