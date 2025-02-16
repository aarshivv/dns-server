use std::{fs::File, io::Read, net::UdpSocket};

use util::{BytePacketBuffer, DnsPacket, DnsQuestion, QueryType, ResultCode};

mod util;

fn lookup(qname: &str, qtype: QueryType) -> Result<DnsPacket, String> {
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

    DnsPacket::from_buffer(&mut res_buffer)
}

fn handle_query(socket: &UdpSocket) -> Result<(), String> {
    let mut req_buffer = BytePacketBuffer::new();

    let (a, src) = socket
        .recv_from(&mut req_buffer.buf)
        .map_err(|e| e.to_string())?;

    let mut request = DnsPacket::from_buffer(&mut req_buffer)?;

    let mut packet = DnsPacket::new();
    packet.header.id = request.header.id;
    packet.header.recursion_desired = true;
    packet.header.recursion_available = true;
    packet.header.response = true;

    if let Some(question) = request.questions.pop() {
        println!("Recieved query: {question:?}");

        if let Ok(result) = lookup(&question.name, question.qtype) {
            packet.questions.push(question);
            packet.header.rescode = result.header.rescode;

            for rec in result.answers {
                println!("answer: {rec:?}");
                packet.answers.push(rec);
            }

            for rec in result.authorities {
                println!("Auth: {rec:?}");
                packet.authorities.push(rec);
            }

            for rec in result.resources {
                println!("Resources: {rec:?}");
                packet.resources.push(rec);
            }
        } else {
            packet.header.rescode = ResultCode::SERVFAIL;
        }
    } else {
        packet.header.rescode = ResultCode::FORMERR;
    }

    let mut res_buffer = BytePacketBuffer::new();
    packet.write(&mut res_buffer)?;

    let len = res_buffer.pos();
    let data = res_buffer.get_range(0, len)?;

    socket.send_to(data, src).map_err(|e| e.to_string())?;

    Ok(())
}

fn main() -> Result<(), String> {
    // let mut f = File::open("response_packet.txt").map_err(|e| e.to_string())?;
    // let mut buffer = BytePacketBuffer::new();
    // f.read(&mut buffer.buf).map_err(|e| e.to_string())?;

    // let packet = DnsPacket::from_buffer(&mut buffer)?;
    // println!("{packet:#?}");

    let qname = "yahoo.com";
    let qtype = QueryType::MX;

    let socket = UdpSocket::bind(("0.0.0.0", 2053)).map_err(|e| e.to_string())?;

    loop {
        match handle_query(&socket) {
            Ok(_) => {}
            Err(e) => eprintln!("an error coccured: {e}"),
        }
    }
}
