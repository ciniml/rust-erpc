# std-transport

A tranport layer implementation using `std::io`.
`IoTransport` can accept a trait bound `std::io::Read + std::io::Write`, e.g., `std::net::Stream`, `SerialPort` struct in `serialport` crate, and so on.

## How to run serial example with Wio Terminal

Instruction for Ubuntu.

Clone this repository.

```sh
git clone https://github.com/ciniml/rust-erpc.git
cd rust-erpc
```

Write the passthrough firmware to Wio Terminal.

```sh
# Wio Terminal should be in boot loader mode
cp test/wioterminal_passthrough/wioterminal_passthrough.uf2 /media/<user>/Arduino/
```

Run serial example.

```sh
$ cargo run --example serial
# ...
Sending request... Ok.
Receiving response... Ok. response = 1, 1, 1, false
        result = 5
Sending request... Ok.
Receiving response... Ok. response = 1, 1, 2, false
        result = 5
# ...
```
