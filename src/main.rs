use std::{fs::File, io::Read, net::UdpSocket};

use util::{BytePacketBuffer, DnsPacket, DnsQuestion, QueryType};

mod util;

fn main() -> Result<(), String> {
    // let mut f = File::open("response_packet.txt").map_err(|e| e.to_string())?;
    // let mut buffer = BytePacketBuffer::new();
    // f.read(&mut buffer.buf).map_err(|e| e.to_string())?;

    // let packet = DnsPacket::from_buffer(&mut buffer)?;
    // println!("{packet:#?}");

    let qname = "yahoo.com";
    let qtype = QueryType::MX;

    let server = ("1.1.1.1", 53);

    let socket = UdpSocket::bind(("0.0.0.0", 43210)).map_err(|e| e.to_string())?;

    let mut packet = DnsPacket::new();

    packet.header.id = 6666;
    packet.header.questions = 1;
    packet.header.recursion_desired = true;

    packet
        .questions
        .push(DnsQuestion::new(qname.to_string(), qtype));

    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer)?;

    socket
        .send_to(&req_buffer.buf[0..req_buffer.pos], server)
        .map_err(|e| e.to_string())?;

    let mut res_buffer = BytePacketBuffer::new();
    socket
        .recv_from(&mut res_buffer.buf)
        .map_err(|e| e.to_string())?;

    let res_packet = DnsPacket::from_buffer(&mut res_buffer)?;

    println!("RESULT: {res_packet:#?}");

    Ok(())
}
