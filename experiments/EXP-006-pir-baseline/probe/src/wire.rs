#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WireError {
    Malformed,
    BindingMismatch,
    ShapeMismatch,
    Limit,
}
impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "experimental PIR message: {self:?}")
    }
}
impl std::error::Error for WireError {}
const MAX_CELLS: usize = 65536;
const MAX_FRAME: usize = 1024 * 1024;
fn check_widths(widths: &[usize]) -> Result<(), WireError> {
    if widths.is_empty() || widths.len() > 2 || widths.iter().any(|n| !(1..=MAX_CELLS).contains(n))
    {
        return Err(WireError::Limit);
    }
    Ok(())
}
pub fn encode(
    magic: [u8; 4],
    binding: [u8; 32],
    vectors: &[Vec<u32>],
) -> Result<Vec<u8>, WireError> {
    check_widths(&vectors.iter().map(Vec::len).collect::<Vec<_>>())?;
    let mut out = Vec::new();
    out.extend(magic);
    out.extend(binding);
    out.extend((vectors.len() as u32).to_be_bytes());
    for vector in vectors {
        out.extend((vector.len() as u32).to_be_bytes());
        for cell in vector {
            out.extend(cell.to_be_bytes());
        }
    }
    Ok(out)
}
pub fn decode(
    frame: &[u8],
    magic: [u8; 4],
    binding: [u8; 32],
    widths: &[usize],
) -> Result<Vec<Vec<u32>>, WireError> {
    check_widths(widths)?;
    if frame.len() > MAX_FRAME {
        return Err(WireError::Limit);
    }
    if frame.get(..4) != Some(magic.as_slice()) {
        return Err(WireError::Malformed);
    }
    let received = frame.get(4..36).ok_or(WireError::Malformed)?;
    if received != binding {
        return Err(WireError::BindingMismatch);
    }
    let mut input = frame.get(36..).ok_or(WireError::Malformed)?;
    if take_u32(&mut input)? as usize != widths.len() {
        return Err(WireError::ShapeMismatch);
    }
    let mut out = Vec::with_capacity(widths.len());
    for width in widths {
        if take_u32(&mut input)? as usize != *width {
            return Err(WireError::ShapeMismatch);
        }
        let byte_length = width * 4; // Width is bounded before arithmetic/allocation.
        let bytes = input.get(..byte_length).ok_or(WireError::Malformed)?;
        out.push(
            bytes
                .chunks_exact(4)
                .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
                .collect(),
        );
        input = &input[byte_length..];
    }
    if !input.is_empty() {
        return Err(WireError::Malformed);
    }
    Ok(out)
}
fn take_u32(input: &mut &[u8]) -> Result<u32, WireError> {
    let bytes = input.get(..4).ok_or(WireError::Malformed)?;
    let out = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    *input = &input[4..];
    Ok(out)
}
