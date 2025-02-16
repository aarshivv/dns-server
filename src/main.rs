use std::sync::Arc;

use buffer::BytePacketBuffer;
use dashmap::DashMap;
use dns::{DnsQuestion, DnsRecord};
use once_cell::sync::OnceCell;
use tokio::{net::UdpSocket, task};
use util::handle_query;

mod buffer;
mod dns;
mod util;

static DNS_CACHE: OnceCell<Arc<DashMap<DnsQuestion, Vec<DnsRecord>>>> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), String> {
    let socket = Arc::new(
        UdpSocket::bind(("0.0.0.0", 2053))
            .await
            .map_err(|e| e.to_string())?,
    );

    println!("Starting DNS server at port 2053");
    DNS_CACHE
        .set(Arc::new(DashMap::new()))
        .expect("ERROR SETTING UP CACHE");

    loop {
        let mut req_buffer = BytePacketBuffer::new();

        let (_, src) = socket
            .recv_from(&mut req_buffer.buf)
            .await
            .map_err(|e| e.to_string())?;

        let socket_clone = socket.clone();
        task::spawn(async move {
            match handle_query(&mut req_buffer).await {
                Ok(mut res_buffer) => {
                    let len = res_buffer.pos();
                    let data = res_buffer.get_range(0, len).unwrap();
                    socket_clone
                        .send_to(data, src)
                        .await
                        .map_err(|e| e.to_string())
                        .unwrap();
                }
                Err(e) => eprintln!("an error coccured: {e}"),
            };
        });
    }
}
