use std::{collections::HashMap, future::Future, net::Ipv4Addr, pin::Pin, time::Duration};

use tokio::{net::UdpSocket, time::timeout};

use crate::{
    buffer::BytePacketBuffer,
    dns::{DnsPacket, DnsQuestion, DnsRecord, QueryType, ResultCode},
};

#[derive(Debug, Default)]
pub struct DnsServer {
    cache: HashMap<DnsQuestion, Vec<DnsRecord>>,
}

impl DnsServer {
    fn check_cache(&self, question: &DnsQuestion) -> Option<&Vec<DnsRecord>> {
        self.cache.get(question)
    }

    fn insert_answer(&mut self, question: DnsQuestion, answers: Vec<DnsRecord>) {
        self.cache.insert(question, answers);
    }

    pub async fn handle_query(
        &mut self,
        req_buffer: &mut BytePacketBuffer,
    ) -> Result<BytePacketBuffer, String> {
        let mut request = DnsPacket::from_buffer(req_buffer)?;

        let mut packet = DnsPacket::new();
        packet.header.id = request.header.id;
        packet.header.recursion_desired = true;
        packet.header.recursion_available = true;
        packet.header.response = true;

        if let Some(question) = request.questions.pop() {
            if let Some(answers) = self.check_cache(&question) {
                println!("Found in cache: returning");
                packet.questions.push(question);
                packet.header.rescode = ResultCode::NOERROR;

                for rec in answers {
                    packet.answers.push(rec.clone());
                }
            } else if let Ok(result) = recursive_lookup(&question.name, question.qtype).await {
                packet.questions.push(question.clone());
                packet.header.rescode = result.header.rescode;

                if result.header.rescode.eq(&ResultCode::NOERROR) && !result.answers.is_empty() {
                    self.insert_answer(question, result.answers.clone());
                }

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

        Ok(res_buffer)
    }
}

fn recursive_lookup<'a>(
    qname: &'a str,
    qtype: QueryType,
) -> Pin<Box<dyn Future<Output = Result<DnsPacket, String>> + 'a>> {
    Box::pin(async move {
        // Using one of the root server from the global root server.
        let mut ns = "198.41.0.4".parse::<Ipv4Addr>().unwrap();

        loop {
            println!("atempting lookup of {qtype:?} {qname} with ns {ns}");

            let ns_copy = ns;

            let server = (ns_copy, 53);
            let response = lookup(qname, qtype, server).await?;

            if !response.answers.is_empty() && response.header.rescode == ResultCode::NOERROR {
                return Ok(response);
            }

            if response.header.rescode == ResultCode::NXDOMAIN {
                return Ok(response);
            }

            if let Some(new_ns) = response.get_resolved_ns(qname) {
                ns = new_ns;

                continue;
            }

            let new_ns_name = match response.get_unresolved_ns(qname) {
                Some(x) => x,
                None => return Ok(response),
            };

            let recursive_response = recursive_lookup(new_ns_name, QueryType::A).await?;

            if let Some(new_ns) = recursive_response.get_random_a() {
                ns = new_ns;
            } else {
                return Ok(response);
            }
        }
    })
}

async fn lookup(
    qname: &str,
    qtype: QueryType,
    server: (Ipv4Addr, u16),
) -> Result<DnsPacket, String> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| e.to_string())?;

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
        .await
        .map_err(|e| e.to_string())?;

    let mut res_buffer = BytePacketBuffer::new();

    match timeout(
        Duration::from_secs(5),
        socket.recv_from(&mut res_buffer.buf),
    )
    .await
    {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => return Err(e.to_string()),
        Err(_) => {
            eprintln!("DNS query timed out!");
            let mut resp_packet = DnsPacket::new();
            resp_packet.header.id = packet.header.id;
            resp_packet.header.recursion_desired = packet.header.recursion_desired;
            resp_packet.header.recursion_available = true;
            resp_packet.header.response = true;
            resp_packet
                .questions
                .push(DnsQuestion::new(qname.to_string(), qtype));

            resp_packet.header.rescode = ResultCode::NXDOMAIN;
            return Ok(resp_packet);
        }
    };

    DnsPacket::from_buffer(&mut res_buffer)
}
