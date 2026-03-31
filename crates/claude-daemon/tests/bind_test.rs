use std::net::SocketAddr;

#[test]
fn test_bind_address_parsing_valid() {
    let addr: SocketAddr = "127.0.0.1:7890".parse().unwrap();
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_eq!(addr.port(), 7890);

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    assert_eq!(addr.ip().to_string(), "0.0.0.0");
    assert_eq!(addr.port(), 8080);
}

#[test]
fn test_bind_address_parsing_invalid() {
    let result: Result<SocketAddr, _> = "not-an-address:7890".parse();
    assert!(result.is_err());
}
