use crate::video::error::Error;

#[allow(non_camel_case_types)]
#[repr(u8)]
pub(crate) enum NalType {
    NonIDR = 1,
    IDR = 5,
    SPS = 7,
    PPS = 8,
    STAP_A = 24,
    FU_A = 28,
    Unknown = 255,
}

impl From<u8> for NalType {
    fn from(value: u8) -> Self {
        match value {
            1 => NalType::NonIDR,
            5 => NalType::IDR,
            7 => NalType::SPS,
            8 => NalType::PPS,
            24 => NalType::STAP_A,
            28 => NalType::FU_A,
            _ => NalType::Unknown,
        }
    }
}

pub(crate) const NAL_PREFIX_CODE: [u8; 4] = [0, 0, 0, 1];

pub fn convert_payload_to_nal_units(
    payload: &[u8],
    nal_buffer: &mut Vec<u8>,
) -> Option<Vec<Vec<u8>>> {
    if payload.is_empty() {
        return None;
    }

    let nal_header = payload[0];
    let nal_type = NalType::from(nal_header & 0x1F);

    match nal_type {
        NalType::STAP_A => convert_stap_a_to_nal_units(&payload),
        NalType::FU_A => convert_fu_a_to_nal_units(&payload, nal_buffer),
        _ => Some(vec![add_prefix_to_nal_unit(&payload)]),
    }
}

fn add_prefix_to_nal_unit(slice: &[u8]) -> Vec<u8> {
    let mut nal_unit = NAL_PREFIX_CODE.to_vec();
    nal_unit.extend_from_slice(slice);
    nal_unit
}

fn convert_stap_a_to_nal_units(payload: &[u8]) -> Option<Vec<Vec<u8>>> {
    let mut nal_units = Vec::new();

    let mut offset = 1;
    while offset + 2 <= payload.len() {
        let size = u16::from_be_bytes([payload[offset], payload[offset + 1]]) as usize;
        offset += 2;

        if size == 0 || offset + size > payload.len() {
            break;
        }

        let nal_unit = add_prefix_to_nal_unit(&payload[offset..offset + size]);
        nal_units.push(nal_unit);

        offset += size;
    }

    Some(nal_units)
}

fn convert_fu_a_to_nal_units(payload: &[u8], nal_buffer: &mut Vec<u8>) -> Option<Vec<Vec<u8>>> {
    if payload.len() < 2 {
        return None;
    }

    let nal_header = payload[0];
    let fu_header = payload[1];

    let start = fu_header & 0x80 != 0;
    if start {
        let reconstructed_nal_header = (nal_header & 0xE0) | (fu_header & 0x1F);
        nal_buffer.clear();
        nal_buffer.push(reconstructed_nal_header);
    }

    nal_buffer.extend_from_slice(&payload[2..]);

    let end = fu_header & 0x40 != 0;
    if end {
        Some(vec![add_prefix_to_nal_unit(&nal_buffer)])
    } else {
        None
    }
}

pub(crate) fn split_prefix_code(nal_unit: &[u8]) -> Result<(NalType, &[u8]), Error> {
    if nal_unit.starts_with(&[0, 0, 1]) {
        Ok((NalType::from(nal_unit[3] & 0x1F), &nal_unit[3..]))
    } else if nal_unit.starts_with(&[0, 0, 0, 1]) {
        Ok((NalType::from(nal_unit[4] & 0x1F), &nal_unit[4..]))
    } else {
        Err(Error::MalformedNalUnit)
    }
}
