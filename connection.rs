use core::comm::Port;
use std::net::tcp::{TcpErrData, TcpConnectErrData, TcpSocket};
use std::net::ip::IpAddr;

pub type ReadPort = Port<Result<~[u8], TcpErrData>>;

/**
An abstract client socket connection. This mirrors the bits
of the net::tcp::TcpSocket interface that we care about
while letting us have additional implementations for
mocking
*/
pub trait Connection {
    fn write_(data: ~[u8]) -> Result<(), TcpErrData>;
    fn read_start_() -> Result<@ReadPort, TcpErrData>;
    fn read_stop_(read_port: @ReadPort) -> Result<(), TcpErrData>;
}

pub trait ConnectionFactory<C: Connection> {
    fn connect(ip: IpAddr, port: uint) -> Result<C, TcpConnectErrData>;
}

impl Connection for TcpSocket {
    fn write_(data: ~[u8]) -> Result<(), TcpErrData> {
        use std::net::tcp::TcpSocket;
        self.write(data)
    }

    fn read_start_() -> Result<@ReadPort, TcpErrData> {
        use std::net::tcp::TcpSocket;
        self.read_start()
    }

    fn read_stop_(read_port: @ReadPort) -> Result<(), TcpErrData> {
        use std::net::tcp::TcpSocket;
        self.read_stop()
    }
}

pub enum UvConnectionFactory {
    UvConnectionFactory
}

impl ConnectionFactory<TcpSocket> for UvConnectionFactory {
    fn connect(ip: IpAddr, port: uint) -> Result<TcpSocket, TcpConnectErrData> {
        use std::uv_global_loop;
        use std::net::tcp::connect;
        let iotask = uv_global_loop::get();
        return connect(copy ip, port, &iotask);
    }
}

pub struct MockConnection {
    write_fn: fn@(~[u8]) -> Result<(), TcpErrData>,
    read_start_fn: fn@() -> Result<@ReadPort, TcpErrData>,
    read_stop_fn: fn@(-port: @ReadPort) -> Result<(), TcpErrData>
}

impl Connection for MockConnection {
    fn write_(data: ~[u8]) -> Result<(), TcpErrData> {
        (self.write_fn)(data)
    }

    fn read_start_() -> Result<@ReadPort, TcpErrData> {
        (self.read_start_fn)()
    }

    fn read_stop_(read_port: @ReadPort) -> Result<(), TcpErrData> {
        (self.read_stop_fn)(read_port)
    }
}

pub struct MockConnectionFactory {
    connect_fn: fn@(IpAddr, uint) -> Result<MockConnection, TcpConnectErrData>
}

impl ConnectionFactory<MockConnection> for MockConnectionFactory {
    fn connect(ip: IpAddr, port: uint) -> Result<MockConnection, TcpConnectErrData> {
        (self.connect_fn)(ip, port)
    }
}
