//! Embedded bitmap font for text rendering.
//!
//! Frustum v0.1 uses a single embedded monospace font.
//! ASCII printable characters (0x20-0x7E) are supported.

/// Font atlas dimensions.
pub const ATLAS_WIDTH: u32 = 128;
pub const ATLAS_HEIGHT: u32 = 64;

/// Character cell dimensions in the atlas.
pub const CHAR_WIDTH: u32 = 8;
pub const CHAR_HEIGHT: u32 = 8;

/// Characters per row in the atlas (128 / 8 = 16).
pub const CHARS_PER_ROW: u32 = ATLAS_WIDTH / CHAR_WIDTH;

/// First printable ASCII character.
pub const FIRST_CHAR: u8 = 0x20; // Space

/// Last printable ASCII character.
pub const LAST_CHAR: u8 = 0x7E; // Tilde

/// Number of characters in the font.
pub const CHAR_COUNT: u32 = (LAST_CHAR - FIRST_CHAR + 1) as u32; // 95 characters

/// Get UV coordinates for a character.
/// Returns (u0, v0, u1, v1) normalized to [0, 1].
pub fn char_uvs(c: char) -> [f32; 4] {
    let code = c as u8;
    let index = if code >= FIRST_CHAR && code <= LAST_CHAR {
        (code - FIRST_CHAR) as u32
    } else {
        // Unknown character: use '?' (0x3F)
        ('?' as u8 - FIRST_CHAR) as u32
    };

    let col = index % CHARS_PER_ROW;
    let row = index / CHARS_PER_ROW;

    let u0 = (col * CHAR_WIDTH) as f32 / ATLAS_WIDTH as f32;
    let v0 = (row * CHAR_HEIGHT) as f32 / ATLAS_HEIGHT as f32;
    let u1 = ((col + 1) * CHAR_WIDTH) as f32 / ATLAS_WIDTH as f32;
    let v1 = ((row + 1) * CHAR_HEIGHT) as f32 / ATLAS_HEIGHT as f32;

    [u0, v0, u1, v1]
}

/// Generate the font atlas as RGBA pixels.
/// Returns a Vec of ATLAS_WIDTH * ATLAS_HEIGHT * 4 bytes.
pub fn generate_atlas() -> Vec<u8> {
    let mut pixels = vec![0u8; (ATLAS_WIDTH * ATLAS_HEIGHT * 4) as usize];

    // 8x8 bitmap font data for ASCII 0x20-0x7E
    // Each character is 8 bytes, one per row, with bits representing pixels
    let font_data: [u64; 95] = [
        // Space to tilde (0x20 - 0x7E)
        0x0000000000000000, // 0x20 ' '
        0x183C3C1818001800, // 0x21 '!'
        0x6C6C6C0000000000, // 0x22 '"'
        0x6C6CFE6CFE6C6C00, // 0x23 '#'
        0x187EC07C06FC1800, // 0x24 '$'
        0x00C6CC183066C600, // 0x25 '%'
        0x386C3876DCCC7600, // 0x26 '&'
        0x1818300000000000, // 0x27 '''
        0x0C18303030180C00, // 0x28 '('
        0x30180C0C0C183000, // 0x29 ')'
        0x006C38FE386C0000, // 0x2A '*'
        0x0018187E18180000, // 0x2B '+'
        0x0000000000181830, // 0x2C ','
        0x0000007E00000000, // 0x2D '-'
        0x0000000000181800, // 0x2E '.'
        0x060C183060C08000, // 0x2F '/'
        0x7CC6CEDEF6E67C00, // 0x30 '0'
        0x1838781818187E00, // 0x31 '1'
        0x7CC60C3860C6FE00, // 0x32 '2'
        0x7CC6063C06C67C00, // 0x33 '3'
        0x1C3C6CCCFE0C1E00, // 0x34 '4'
        0xFEC0FC0606C67C00, // 0x35 '5'
        0x3C60C0FCC6C67C00, // 0x36 '6'
        0xFEC6060C18181800, // 0x37 '7'
        0x7CC6C67CC6C67C00, // 0x38 '8'
        0x7CC6C67E06067C00, // 0x39 '9'
        0x0018180018180000, // 0x3A ':'
        0x0018180018183000, // 0x3B ';'
        0x0C18306030180C00, // 0x3C '<'
        0x00007E007E000000, // 0x3D '='
        0x6030180C18306000, // 0x3E '>'
        0x7CC60C1818001800, // 0x3F '?'
        0x7CC6DEDEDCC07E00, // 0x40 '@'
        0x386CC6FEC6C6C600, // 0x41 'A'
        0xFCC6C6FCC6C6FC00, // 0x42 'B'
        0x3C66C0C0C0663C00, // 0x43 'C'
        0xF8CCC6C6C6CCF800, // 0x44 'D'
        0xFEC0C0FCC0C0FE00, // 0x45 'E'
        0xFEC0C0FCC0C0C000, // 0x46 'F'
        0x3C66C0CEC6663E00, // 0x47 'G'
        0xC6C6C6FEC6C6C600, // 0x48 'H'
        0x7E18181818187E00, // 0x49 'I'
        0x1E06060606C67C00, // 0x4A 'J'
        0xC6CCD8F0D8CCC600, // 0x4B 'K'
        0xC0C0C0C0C0C0FE00, // 0x4C 'L'
        0xC6EEFED6C6C6C600, // 0x4D 'M'
        0xC6E6F6DECEC6C600, // 0x4E 'N'
        0x7CC6C6C6C6C67C00, // 0x4F 'O'
        0xFCC6C6FCC0C0C000, // 0x50 'P'
        0x7CC6C6C6D6DE7C06, // 0x51 'Q'
        0xFCC6C6FCD8CCC600, // 0x52 'R'
        0x7CC6C07C06C67C00, // 0x53 'S'
        0x7E18181818181800, // 0x54 'T'
        0xC6C6C6C6C6C67C00, // 0x55 'U'
        0xC6C6C6C66C381000, // 0x56 'V'
        0xC6C6C6D6FEEEC600, // 0x57 'W'
        0xC6C66C386CC6C600, // 0x58 'X'
        0x6666663C18181800, // 0x59 'Y'
        0xFE060C1830C0FE00, // 0x5A 'Z'
        0x3C30303030303C00, // 0x5B '['
        0xC06030180C060200, // 0x5C '\'
        0x3C0C0C0C0C0C3C00, // 0x5D ']'
        0x10386CC600000000, // 0x5E '^'
        0x00000000000000FF, // 0x5F '_'
        0x30180C0000000000, // 0x60 '`'
        0x00007C067EC67E00, // 0x61 'a'
        0xC0C0FCC6C6C6FC00, // 0x62 'b'
        0x00007CC6C0C67C00, // 0x63 'c'
        0x06067EC6C6C67E00, // 0x64 'd'
        0x00007CC6FEC07C00, // 0x65 'e'
        0x1C3630FC30303000, // 0x66 'f'
        0x00007EC6C67E067C, // 0x67 'g'
        0xC0C0FCC6C6C6C600, // 0x68 'h'
        0x1800381818183C00, // 0x69 'i'
        0x0C000C0C0C0CCC78, // 0x6A 'j'
        0xC0C0C6CCD8CCC600, // 0x6B 'k'
        0x3818181818183C00, // 0x6C 'l'
        0x0000ECFED6D6C600, // 0x6D 'm'
        0x0000FCC6C6C6C600, // 0x6E 'n'
        0x00007CC6C6C67C00, // 0x6F 'o'
        0x0000FCC6C6FCC0C0, // 0x70 'p'
        0x00007EC6C67E0606, // 0x71 'q'
        0x0000DEC0C0C0C000, // 0x72 'r'
        0x00007CC07C067C00, // 0x73 's'
        0x3030FC3030301C00, // 0x74 't'
        0x0000C6C6C6C67E00, // 0x75 'u'
        0x0000C6C6C66C3800, // 0x76 'v'
        0x0000C6D6D6FE6C00, // 0x77 'w'
        0x0000C66C386CC600, // 0x78 'x'
        0x0000C6C6C67E0678, // 0x79 'y'
        0x0000FE0C3860FE00, // 0x7A 'z'
        0x0E18187018180E00, // 0x7B '{'
        0x1818181818181800, // 0x7C '|'
        0x7018180E18187000, // 0x7D '}'
        0x76DC000000000000, // 0x7E '~'
    ];

    // Render each character into the atlas
    for (char_idx, &bitmap) in font_data.iter().enumerate() {
        let col = char_idx as u32 % CHARS_PER_ROW;
        let row = char_idx as u32 / CHARS_PER_ROW;
        let base_x = col * CHAR_WIDTH;
        let base_y = row * CHAR_HEIGHT;

        // Each row of the character (8 rows)
        for y in 0..8u32 {
            let row_bits = ((bitmap >> (56 - y * 8)) & 0xFF) as u8;
            for x in 0..8u32 {
                let bit = (row_bits >> (7 - x)) & 1;
                let px = base_x + x;
                let py = base_y + y;
                let idx = ((py * ATLAS_WIDTH + px) * 4) as usize;

                if bit == 1 {
                    // White foreground
                    pixels[idx] = 255;
                    pixels[idx + 1] = 255;
                    pixels[idx + 2] = 255;
                    pixels[idx + 3] = 255;
                } else {
                    // Transparent background
                    pixels[idx] = 0;
                    pixels[idx + 1] = 0;
                    pixels[idx + 2] = 0;
                    pixels[idx + 3] = 0;
                }
            }
        }
    }

    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_uvs() {
        // Space should be at (0, 0)
        let uvs = char_uvs(' ');
        assert_eq!(uvs[0], 0.0);
        assert_eq!(uvs[1], 0.0);

        // 'A' is at index 33 (0x41 - 0x20)
        // col = 33 % 16 = 1, row = 33 / 16 = 2
        let uvs = char_uvs('A');
        assert!((uvs[0] - 1.0 / 16.0).abs() < 0.001);
        assert!((uvs[1] - 2.0 / 8.0).abs() < 0.001);
    }

    #[test]
    fn test_generate_atlas() {
        let atlas = generate_atlas();
        assert_eq!(atlas.len(), (ATLAS_WIDTH * ATLAS_HEIGHT * 4) as usize);

        // Check that 'A' has some non-zero pixels
        // 'A' is at index 33, col=1, row=2
        let base_x = 1 * CHAR_WIDTH;
        let base_y = 2 * CHAR_HEIGHT;
        // At least some pixel in 'A' should be non-zero
        let mut has_pixel = false;
        for dy in 0..8 {
            for dx in 0..8 {
                let px = base_x + dx;
                let py = base_y + dy;
                let i = ((py * ATLAS_WIDTH + px) * 4) as usize;
                if atlas[i + 3] > 0 {
                    has_pixel = true;
                    break;
                }
            }
        }
        assert!(has_pixel, "'A' should have visible pixels");
    }

    #[test]
    fn test_unknown_char_fallback() {
        // Non-ASCII should fall back to '?'
        let uvs_unknown = char_uvs('\u{00FF}');
        let uvs_question = char_uvs('?');
        assert_eq!(uvs_unknown, uvs_question);
    }
}
