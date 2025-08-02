pub fn handle_rtp(payload: &[u8], nal_buffer: &mut Vec<u8>) -> Vec<Vec<u8>> {
    let mut nal_units = Vec::new();

    if payload.is_empty() {
        return nal_units;
    }

    let nal_header = payload[0];
    let nal_type = nal_header & 0x1F;

    match nal_type {
        24 => {
            // STAP-A (Single-Time Aggregation Packet)
            let mut offset = 1;
            while offset + 2 <= payload.len() {
                let size = u16::from_be_bytes([payload[offset], payload[offset + 1]]) as usize;
                offset += 2;

                if size == 0 || offset + size > payload.len() {
                    break; // malformed
                }

                let nal = payload[offset..offset + size].to_vec();
                nal_units.push(nal);

                offset += size;
            }
        }

        28 => {
            // FU-A (Fragmentation Unit)
            if payload.len() < 2 {
                return nal_units;
            }

            let fu_header = payload[1];
            let start = fu_header & 0x80 != 0;
            let end = fu_header & 0x40 != 0;

            if start {
                nal_buffer.clear();
                let reconstructed_nal_header = (nal_header & 0xE0) | (fu_header & 0x1F);
                nal_buffer.push(reconstructed_nal_header);
            }

            nal_buffer.extend_from_slice(&payload[3..]);

            if end {
                nal_units.push(nal_buffer.clone());
                nal_buffer.clear();
            }
        }

        _ => {
            // Single NAL unit
            nal_units.push(payload.to_vec());
        }
    }

    nal_units
}
