use crate::ParseError;

type Result<Ok> = std::result::Result<Ok, ParseError>;

pub struct RawFen<'a> {
    pub pieces: &'a str,
    pub active_color: &'a str,
    pub castling: &'a str,
    pub en_passant: &'a str,
    pub halfmove_clock: &'a str,
    pub fullmove_number: &'a str,
}

impl<'a> RawFen<'a> {
    pub fn parse(s: &'a str) -> Result<Self> {
        let parts: Vec<_> = s.split_whitespace().collect();

        let pieces = *Self::field_or(&parts, 0)?;
        let active_color = *Self::field_or(&parts, 1)?;
        let castling = *Self::field_or(&parts, 2)?;
        let en_passant = *Self::field_or(&parts, 3)?;
        let halfmove_clock = *Self::field_or(&parts, 4)?;
        let fullmove_number = *Self::field_or(&parts, 5)?;

        Ok(RawFen {
            pieces,
            active_color,
            castling,
            en_passant,
            halfmove_clock,
            fullmove_number,
        })
    }

    fn field_or<'v, 's>(v: &'v [&'s str], idx: usize) -> Result<&'v &'s str> {
        v.get(idx).ok_or(ParseError::MissingField(idx))
    }
}
