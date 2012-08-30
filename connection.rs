import comm::Port;
import std::net::tcp::{tcp_err_data, tcp_connect_err_data};
import std::net::ip::ip_addr;

type ReadPort = Port<Result<~[u8], tcp_err_data>>;

/**
An abstract client socket connection. This mirrors the bits
of the net::tcp::tcp_socket interface that we care about
while letting us have additional implementations for
mocking
*/
trait Connection {
    fn write_(data: ~[u8]) -> Result<(), tcp_err_data>;
    fn read_start_() -> Result<ReadPort, tcp_err_data>;
    fn read_stop_(-read_port: ReadPort) -> Result<(), tcp_err_data>;
}

trait ConnectionFactory<C: Connection> {
    fn connect(ip: ip_addr, port: uint) -> Result<C, tcp_connect_err_data>;
}

impl tcp_socket : Connection {
    fn write_(data: ~[u8]) -> Result<(), tcp_err_data> {
        import std::net::tcp::tcp_socket;
        self.write(data)
    }

    fn read_start_() -> Result<ReadPort, tcp_err_data> {
        import std::net::tcp::tcp_socket;
        self.read_start()
    }

    fn read_stop_(-read_port: ReadPort) -> Result<(), tcp_err_data> {
        import std::net::tcp::tcp_socket;
        self.read_stop(read_port)
    }
}

enum UvConnectionFactory {
    UvConnectionFactory
}

impl UvConnectionFactory : ConnectionFactory<tcp_socket> {
    fn connect(ip: ip_addr, port: uint) -> Result<tcp_socket, tcp_connect_err_data> {
        import std::uv_global_loop;
        import std::net::tcp::connect;
        let iotask = uv_global_loop::get();
        return connect(copy ip, port, iotask);
    }
}

type MockConnection = {
    write_fn: fn@(~[u8]) -> Result<(), tcp_err_data>,
    read_start_fn: fn@() -> Result<ReadPort, tcp_err_data>,
    read_stop_fn: fn@(-ReadPort) -> Result<(), tcp_err_data>
};

impl MockConnection : Connection {
    fn write_(data: ~[u8]) -> Result<(), tcp_err_data> {
        self.write_fn(data)
    }

    fn read_start_() -> Result<ReadPort, tcp_err_data> {
        self.read_start_fn()
    }

    fn read_stop_(-read_port: ReadPort) -> Result<(), tcp_err_data> {
        self.read_stop_fn(read_port)
    }
}

type MockConnectionFactory = {
    connect_fn: fn@(ip_addr, uint) -> Result<MockConnection, tcp_connect_err_data>
};

impl MockConnectionFactory : ConnectionFactory<MockConnection> {
    fn connect(ip: ip_addr, port: uint) -> Result<MockConnection, tcp_connect_err_data> {
        self.connect_fn(ip, port)
    }
}
