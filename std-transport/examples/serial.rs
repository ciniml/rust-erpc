/// An example to use `serialport` crate as a transport layer.

use serialport::open;

use std_transport::IoTransport;
use rust_erpc::framed_transport::{BasicFramedTransport, FramedTransportError};
use rust_erpc::request::{Request, Response};
use rust_erpc::{
    codec::{BasicCodecFactory, Codec},
    request::RequestResponseError,
};

fn main() {
    let mut port = open("/dev/ttyACM0").expect("Failed to open serial port.");
    port.set_baud_rate(1843200)
        .expect("Failed to set baud rate");

    let io_transport = IoTransport::new(port);
    let mut transport = BasicFramedTransport::new(io_transport);
    let mut frame_buffer = [0u8; 1024];

    let mut sequence = 0u32;
    'main: loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        sequence += 1;

        print!("Sending request... ");
        let request = Request::new(1, 1, sequence, false);
        let result = request.send_request(
            &mut transport,
            &mut frame_buffer,
            BasicCodecFactory::new(),
            |codec| {
                //codec.write_u32(1234)?;
                //codec.write_binary(&[0x5a; 128])?;
                Ok(())
            },
        );
        if let Err(err) = result {
            println!("Error: {:?}", err);
            continue 'main;
        }
        println!("Ok.");

        print!("Receiving response... ");
        loop {
            let result = Response::receive_response(
                &mut transport,
                &mut frame_buffer,
                BasicCodecFactory::new(),
            );
            match result {
                Ok((response, mut codec)) => {
                    println!(
                        "Ok. response = {}, {}, {}, {}",
                        response.service,
                        response.request,
                        response.sequence,
                        response.is_notification
                    );
                    if let Ok(result) = codec.read_u32() {
                        println!("\tresult = {}", result);
                    } else {
                        println!("\tno result...");
                    }
                    break;
                }
                Err(err) => {
                    if let RequestResponseError::FramedTransportError(
                        FramedTransportError::UnderlyingError(underlying_error),
                    ) = &err
                    {
                        if underlying_error.kind() == std::io::ErrorKind::WouldBlock {
                            continue;
                        }
                    } else {
                    }
                    println!("Error: {:?}", err);
                    continue 'main;
                }
            }
        }
    }
}
