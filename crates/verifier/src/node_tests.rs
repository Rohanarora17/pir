use super::node::validate;
use alloy_rlp::Header;

#[test]
fn branch_values_cannot_trigger_the_single_high_byte_decoder_ambiguity() {
    for byte in [0x80, 0xff] {
        let mut payload = vec![0x80; 16]; // Empty child references.
        payload.extend(alloy_rlp::encode(&[byte][..]));
        let mut node = Vec::new();
        Header {
            list: true,
            payload_length: payload.len(),
        }
        .encode(&mut node);
        node.extend(payload);
        assert!(validate(&node).is_err());
    }
}
