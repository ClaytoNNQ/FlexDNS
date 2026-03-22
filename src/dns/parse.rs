use std::time::Duration;

pub fn parse(packet: &[u8]) -> Duration {
    let mut pkt = packet;

    // TCP length prefix
    if pkt.len() > 2 {
        let len = u16::from_be_bytes([pkt[0], pkt[1]]) as usize;
        if len + 2 == pkt.len() {
            pkt = &pkt[2..];
        }
    }

    if pkt.len() < 12 {
        return Duration::ZERO;
    }

    let qdcount = u16::from_be_bytes([pkt[4], pkt[5]]) as usize;
    let ancount = u16::from_be_bytes([pkt[6], pkt[7]]) as usize;

    let mut pos = 12;

    // skip questions
    for _ in 0..qdcount {
        pos = skip_name(pkt, pos);
        pos += 4; // type + class
    }

    if ancount == 0 {
        return Duration::ZERO;
    }

    let mut min_ttl = u32::MAX;

    for _ in 0..ancount {
        pos = skip_name(pkt, pos);

        if pos + 10 > pkt.len() {
            break;
        }

        let _rtype = u16::from_be_bytes([pkt[pos], pkt[pos + 1]]);
        let _class = u16::from_be_bytes([pkt[pos + 2], pkt[pos + 3]]);
        let ttl = u32::from_be_bytes([
            pkt[pos + 4],
            pkt[pos + 5],
            pkt[pos + 6],
            pkt[pos + 7],
        ]);
        let rdlen = u16::from_be_bytes([pkt[pos + 8], pkt[pos + 9]]) as usize;

        min_ttl = min_ttl.min(ttl);

        pos += 10 + rdlen;
    }

    if min_ttl == u32::MAX {
        Duration::ZERO
    } else {
        Duration::from_secs(min_ttl as u64)
    }
}

fn skip_name(pkt: &[u8], mut pos: usize) -> usize {
    loop {
        if pos >= pkt.len() {
            return pos;
        }

        let len = pkt[pos];

        // pointer
        if len & 0xC0 == 0xC0 {
            return pos + 2;
        }

        // end
        if len == 0 {
            return pos + 1;
        }

        pos += len as usize + 1;
    }
}