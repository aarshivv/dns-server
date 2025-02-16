use buffer::BytePacketBuffer;
use tokio::net::UdpSocket;
use util::DnsServer;

mod buffer;
mod dns;
mod util;

#[tokio::main]
async fn main() -> Result<(), String> {
    let socket = UdpSocket::bind(("0.0.0.0", 2053))
        .await
        .map_err(|e| e.to_string())?;

    println!("Starting DNS server at port 2053");

    let mut dns_server = DnsServer::default();

    loop {
        let mut req_buffer = BytePacketBuffer::new();

        let (_, src) = socket
            .recv_from(&mut req_buffer.buf)
            .await
            .map_err(|e| e.to_string())?;

        match dns_server.handle_query(&mut req_buffer).await {
            Ok(mut res_buffer) => {
                let len = res_buffer.pos();
                let data = res_buffer.get_range(0, len)?;
                socket.send_to(data, src).await.map_err(|e| e.to_string())?;
            }
            Err(e) => eprintln!("an error coccured: {e}"),
        }
    }
}
