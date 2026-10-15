//! Bounded structural admission before eth_trie's recursive decoder.
//! This does not follow hashed paths or decide membership; eth_trie does that.
use alloy_rlp::Header;

pub(super) fn validate(encoded: &[u8]) -> Result<(), ()> {
    validate_node(encoded, 0)
}

// A secure Ethereum key has 64 nibbles. Bound even irrelevant inline subtrees;
// configurable byte limits alone do not safely bound recursive decoder depth.
const MAX_INLINE_DEPTH: usize = 64;

fn item<'a>(input: &mut &'a [u8]) -> Result<(bool, &'a [u8], &'a [u8]), ()> {
    let original = *input;
    let header = Header::decode(input).map_err(|_| ())?;
    let (payload, rest) = input.split_at(header.payload_length);
    *input = rest;
    Ok((
        header.list,
        payload,
        &original[..original.len() - rest.len()],
    ))
}

fn string(encoded: &[u8]) -> Result<&[u8], ()> {
    let mut input = encoded;
    let (list, payload, _) = item(&mut input)?;
    if list || !input.is_empty() {
        return Err(());
    }
    Ok(payload)
}

fn value(encoded: &[u8]) -> Result<(), ()> {
    let payload = string(encoded)?;
    // eth_trie 0.6.1 retains the outer prefix for a one-byte payload >= 0x80.
    // Such bytes cannot encode a valid Ethereum account or nonzero storage integer:
    // 0x80 is stored zero; other high single bytes cannot encode either valid value type.
    // Reject before recovery rather than allowing reinterpretation as another integer.
    if payload.len() == 1 && payload[0] >= 0x80 {
        return Err(());
    }
    Ok(())
}

fn child(encoded: &[u8], depth: usize) -> Result<(), ()> {
    let mut input = encoded;
    let (list, payload, _) = item(&mut input)?;
    if !input.is_empty() {
        return Err(());
    }
    if list {
        // Nodes of 32 bytes or more must be referenced by hash rather than embedded.
        if encoded.len() >= 32 {
            return Err(());
        }
        validate_node(encoded, depth + 1)
    } else if payload.is_empty() || payload.len() == 32 {
        Ok(())
    } else {
        Err(())
    }
}

fn validate_node(encoded: &[u8], depth: usize) -> Result<(), ()> {
    if depth > MAX_INLINE_DEPTH {
        return Err(());
    }
    let mut input = encoded;
    let (is_list, mut payload, _) = item(&mut input)?;
    if !input.is_empty() {
        return Err(());
    }
    if !is_list {
        return if payload.is_empty() { Ok(()) } else { Err(()) };
    }

    // Fixed capacity bounds both work and allocation for branch/path nodes.
    let mut fields = [&[][..]; 17];
    let mut count = 0;
    while !payload.is_empty() {
        if count == fields.len() {
            return Err(());
        }
        fields[count] = item(&mut payload)?.2;
        count += 1;
    }
    match count {
        2 => {
            let compact = string(fields[0])?;
            let first = *compact.first().ok_or(())?;
            let flag = first >> 4;
            if flag > 3 || (flag & 1 == 0 && first & 15 != 0) {
                return Err(());
            }
            let nibbles = (compact.len() - 1) * 2 + usize::from(flag & 1);
            if nibbles > 64 {
                return Err(());
            }
            if flag & 2 != 0 {
                value(fields[1])?;
            } else {
                // Empty extensions can panic in upstream compact-path handling.
                if nibbles == 0 {
                    return Err(());
                }
                child(fields[1], depth)?;
            }
        }
        17 => {
            for field in &fields[..16] {
                child(field, depth)?;
            }
            value(fields[16])?;
        }
        _ => return Err(()),
    }
    Ok(())
}
