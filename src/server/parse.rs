pub fn parse_cache_key(query_bytes: &[u8]) -> Vec<u8> {
    let mut key = Vec::new();

    // QDCOUNT
    let qdcount = ((query_bytes[4] as u16) << 8) | query_bytes[5] as u16;
    if qdcount == 0 { return key; }

    // Question section
    let mut idx = 12;
    while idx < query_bytes.len() {
        let len = query_bytes[idx] as usize;
        if len == 0 {
            idx += 1;
            break;
        }
        let label_bytes = &query_bytes[idx + 1..idx + 1 + len];
        let lower_label = label_bytes.iter().map(|b| b.to_ascii_lowercase()).collect::<Vec<u8>>();

        key.push(len as u8);
        key.extend_from_slice(&lower_label);

        idx += len + 1;
    }
    // QTYPE and QCLASS
    if idx + 4 <= query_bytes.len() {
        key.extend_from_slice(&query_bytes[idx..idx+4]);
    }
    return key;
}