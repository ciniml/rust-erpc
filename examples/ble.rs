use std::{net::TcpStream, time::Duration};

use rust_erpc::cursor::BufferCursor;
use rust_erpc::framed_transport::{BasicFramedTransport, FramedTransportError, IoTransport};
use rust_erpc::request::{MessageType, Request, Response};
use rust_erpc::{
    codec::{BasicCodecFactory, Codec},
    request::RequestResponseError,
};

fn main() {
    let remote = "127.0.0.1:5555"
        .parse()
        .expect("Failed to parse the remote address.");
    let mut stream =
        TcpStream::connect_timeout(&remote, Duration::from_secs(1)).expect("Could not connect.");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();

    let io_transport = IoTransport::new(stream);
    let mut transport = BasicFramedTransport::new(io_transport);
    let mut frame_buffer = [0u8; 1024];

    'main: loop {
        print!("Sending request... ");
        let request = Request::new(2, 1, 0, false);
        let result = request.send_request(
            &mut transport,
            &mut frame_buffer,
            BasicCodecFactory::new(),
            |codec| {
                codec.write_u32(1234)?;
                codec.write_binary(&[0x5a; 128])?;
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
