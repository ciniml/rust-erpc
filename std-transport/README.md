# std-transport

A tranport layer implementation using `std::io`.
`IoTransport` can accept a trait bound `std::io::Read + std::io::Write`, e.g., `std::net::Stream`, `SerialPort` struct in `serialport` crate, and so on.
