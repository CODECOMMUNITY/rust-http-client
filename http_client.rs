import result::result;
import std::net::ip::{
    get_addr, format_addr, ipv4, ipv6, ip_addr,
    ip_get_addr_err
};
import std::net::tcp::{connect, tcp_socket};
import std::uv_global_loop;
import comm::{methods};
import connection::{
    Connection, ConnectionFactory, UvConnectionFactory
};

const timeout: uint = 2000;

/**
A quick hack URI type
*/
type Uri = {
    host: str,
    path: str
};

/// HTTP status codes
enum StatusCode {
    StatusOk = 200,
    StatusUnknown
}

/// HTTP request error conditions
enum RequestError {
    ErrorDnsResolution,
    ErrorConnect,
    ErrorMisc
}

/// Request 
enum RequestEvent {
    Status(StatusCode),
    Payload(~mut option<~[u8]>),
    Error(RequestError)
}

type DnsResolver = fn@(host: str) -> result<~[ip_addr], ip_get_addr_err>;

fn uv_dns_resolver() -> DnsResolver {
    |host| {
        let iotask = uv_global_loop::get();
        get_addr(host, iotask)
    }
}

fn uv_http_request(+uri: Uri) -> HttpRequest<tcp_socket, UvConnectionFactory> {
    HttpRequest(uv_dns_resolver(), UvConnectionFactory, uri)
}

class HttpRequest<C: Connection, CF: ConnectionFactory<C>> {

    let resolve_ip_addr: DnsResolver;
    let connection_factory: CF;
    let uri: Uri;

    new(resolver: DnsResolver, +connection_factory: CF, +uri: Uri) {
        self.resolve_ip_addr = resolver;
        self.connection_factory = connection_factory;
        self.uri = uri;
    }

    fn begin(cb: fn(+RequestEvent)) {
        #debug("http_client: looking up uri %?", self.uri);
        let ip_addr = {
            let ip_addrs = self.resolve_ip_addr(self.uri.host);
            if ip_addrs.is_ok() {
                let ip_addrs = result::unwrap(ip_addrs);
                // FIXME: This log crashes
                //#debug("http_client: got IP addresses for %?: %?", self.uri, ip_addrs);
                if ip_addrs.is_not_empty() {
                    // FIXME: Which address should we really pick?
                    let best_ip = do ip_addrs.find |ip| {
                        alt ip {
                          ipv4(*) { true }
                          ipv6(*) { false }
                        }
                    };

                    if best_ip.is_some() {
                        option::unwrap(best_ip)
                    } else {
                        // FIXME: Need test
                        cb(Error(ErrorMisc));
                        ret;
                    }
                } else {
                    #debug("http_client: got no IP addresses for %?", self.uri);
                    // FIXME: Need test
                    cb(Error(ErrorMisc));
                    ret;
                }
            } else {
                #debug("http_client: DNS lookup failure: %?", ip_addrs.get_err());
                cb(Error(ErrorDnsResolution));
                ret;
            }
        };

        #debug("http_client: using IP %? for %?", format_addr(ip_addr), self.uri);

        let socket = {
            #debug("http_client: connecting to %?", ip_addr);
            let socket = self.connection_factory.connect(copy ip_addr, 80);
            if socket.is_ok() {
                result::unwrap(socket)
            } else {
                #debug("http_client: unable to connect to %?: %?", ip_addr, socket);
                cb(Error(ErrorConnect));
                ret;
            }
        };

        #debug("http_client: got socket for %?", ip_addr);

        let request_header = #fmt("GET %s HTTP/1.0\u000D\u000AHost: %s\u000D\u000A\u000D\u000A",
                                  self.uri.path, self.uri.host);
        #debug("http_client: writing request header: %?", request_header);
        let request_header_bytes = str::bytes(request_header);
        alt socket.write(request_header_bytes) {
          result::ok(*) { }
          result::err(e) {
            // FIXME: Need test
            cb(Error(ErrorMisc));
            ret;
          }
        }

        let read_port = {
            let read_port = socket.read_start();
            if read_port.is_ok() {
                result::unwrap(read_port)
            } else {
                cb(Error(ErrorMisc));
                ret;
            }
        };

        loop {
            let next_data = read_port.recv();

            if next_data.is_ok() {
                let next_data = result::unwrap(next_data);
                let the_payload = Payload(~mut some(next_data));
                cb(the_payload);
            } else {
                #debug("http_client: read error: %?", next_data);

                // This method of detecting EOF is lame
                alt next_data {
                  result::err({err_name: "EOF", _}) {
                    break;
                  }
                  _ {
                    // FIXME: Need tests and error handling
                    socket.read_stop(read_port);
                    cb(Error(ErrorMisc));
                    ret;
                  }
                }
            }
        }
        socket.read_stop(read_port);
    }
}

fn sequence<C: Connection, CF: ConnectionFactory<C>>(request: HttpRequest<C, CF>) -> ~[RequestEvent] {
    let mut events = ~[];
    do request.begin |event| {
        vec::push(events, event)
    }
    ret events;
}

#[test]
fn test_resolve_error() {
    let uri = {
        host: "example.com_not_real",
        path: "/"
    };

    let request = uv_http_request(uri);
    let events = sequence(request);

    assert events == ~[
        Error(ErrorDnsResolution),
    ];
}

#[test]
fn test_connect_error() {
    let uri = {
        // This address is invalid because the first octet
        // of a class A address cannot be 0
        host: "0.42.42.42",
        path: "/"
    };

    let request = uv_http_request(uri);
    let events = sequence(request);

    assert events == ~[
        Error(ErrorConnect),
    ];
}

#[test]
fn test_connect_success() {
    let uri = {
        host: "example.com",
        path: "/"
    };

    let request = uv_http_request(uri);
    let events = sequence(request);

    for events.each |ev| {
        alt ev {
          Error(*) { fail }
          _ { }
        }
    }
}
