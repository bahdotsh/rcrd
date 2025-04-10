use std::collections::HashMap;

// Character bitmap for rendering text
pub type CharBitmap = Vec<Vec<bool>>;

// Scale a bitmap to the desired size
// Scale a bitmap to the desired size
pub fn scale_bitmap(bitmap: &CharBitmap, scale: usize) -> CharBitmap {
    if scale <= 1 {
        return bitmap.clone();
    }

    let mut scaled = Vec::with_capacity(bitmap.len() * scale);

    for row in bitmap {
        let mut scaled_rows = vec![vec![false; row.len() * scale]; scale];

        for (x, &pixel) in row.iter().enumerate() {
            for sy in 0..scale {
                for sx in 0..scale {
                    scaled_rows[sy][x * scale + sx] = pixel;
                }
            }
        }

        scaled.extend(scaled_rows);
    }

    scaled
}

// Create bitmap representations of characters
pub fn create_character_bitmaps() -> HashMap<char, CharBitmap> {
    let mut maps = HashMap::new();

    // Space and special characters
    maps.insert(
        ' ',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '!',
        vec![
            vec![true],
            vec![true],
            vec![true],
            vec![true],
            vec![true],
            vec![false],
            vec![true],
        ],
    );

    maps.insert(
        '"',
        vec![
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '#',
        vec![
            vec![false, true, false, true, false],
            vec![false, true, false, true, false],
            vec![true, true, true, true, true],
            vec![false, true, false, true, false],
            vec![true, true, true, true, true],
            vec![false, true, false, true, false],
            vec![false, true, false, true, false],
        ],
    );

    maps.insert(
        '$',
        vec![
            vec![false, true, true, true, false],
            vec![true, false, true, false, false],
            vec![true, false, true, false, false],
            vec![false, true, true, true, false],
            vec![false, false, true, false, true],
            vec![false, false, true, false, true],
            vec![false, true, true, true, false],
        ],
    );

    maps.insert(
        '%',
        vec![
            vec![true, true, false, false, false],
            vec![true, true, false, false, true],
            vec![false, false, false, true, false],
            vec![false, false, true, false, false],
            vec![false, true, false, false, false],
            vec![true, false, false, true, true],
            vec![false, false, false, true, true],
        ],
    );

    maps.insert(
        '&',
        vec![
            vec![false, true, true, false, false],
            vec![true, false, false, true, false],
            vec![true, false, true, false, false],
            vec![false, true, false, false, false],
            vec![true, false, true, false, true],
            vec![true, false, false, true, false],
            vec![false, true, true, false, true],
        ],
    );

    maps.insert(
        '\'',
        vec![
            vec![true],
            vec![true],
            vec![true],
            vec![false],
            vec![false],
            vec![false],
            vec![false],
        ],
    );

    maps.insert(
        '(',
        vec![
            vec![false, true],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![false, true],
        ],
    );

    maps.insert(
        ')',
        vec![
            vec![true, false],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![true, false],
        ],
    );

    maps.insert(
        '*',
        vec![
            vec![false, false, false],
            vec![true, false, true],
            vec![false, true, false],
            vec![true, true, true],
            vec![false, true, false],
            vec![true, false, true],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '+',
        vec![
            vec![false, false, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        ',',
        vec![
            vec![false, false],
            vec![false, false],
            vec![false, false],
            vec![false, false],
            vec![false, false],
            vec![false, true],
            vec![true, false],
        ],
    );

    maps.insert(
        '-',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![true, true, true],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '.',
        vec![
            vec![false],
            vec![false],
            vec![false],
            vec![false],
            vec![false],
            vec![false],
            vec![true],
        ],
    );

    maps.insert(
        '/',
        vec![
            vec![false, false, false, true],
            vec![false, false, true, false],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![false, true, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
    );

    // Numbers
    maps.insert(
        '0',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, true, true],
            vec![true, true, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        '1',
        vec![
            vec![false, true, false],
            vec![true, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
    );

    maps.insert(
        '2',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![false, false, false, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![true, false, false, false],
            vec![true, true, true, true],
        ],
    );

    maps.insert(
        '3',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![false, false, false, true],
            vec![false, true, true, false],
            vec![false, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        '4',
        vec![
            vec![false, false, true, false],
            vec![false, true, true, false],
            vec![true, false, true, false],
            vec![true, false, true, false],
            vec![true, true, true, true],
            vec![false, false, true, false],
            vec![false, false, true, false],
        ],
    );

    maps.insert(
        '5',
        vec![
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![false, false, false, true],
            vec![false, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        '6',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        '7',
        vec![
            vec![true, true, true, true],
            vec![false, false, false, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![false, true, false, false],
            vec![false, true, false, false],
            vec![false, true, false, false],
        ],
    );

    maps.insert(
        '8',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        '9',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
            vec![false, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        ':',
        vec![
            vec![false],
            vec![true],
            vec![true],
            vec![false],
            vec![true],
            vec![true],
            vec![false],
        ],
    );

    maps.insert(
        ';',
        vec![
            vec![false, false],
            vec![false, true],
            vec![false, true],
            vec![false, false],
            vec![false, true],
            vec![false, true],
            vec![true, false],
        ],
    );

    maps.insert(
        '<',
        vec![
            vec![false, false, true],
            vec![false, true, false],
            vec![true, false, false],
            vec![true, false, false],
            vec![true, false, false],
            vec![false, true, false],
            vec![false, false, true],
        ],
    );

    maps.insert(
        '=',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![true, true, true],
            vec![false, false, false],
            vec![true, true, true],
            vec![false, false, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '>',
        vec![
            vec![true, false, false],
            vec![false, true, false],
            vec![false, false, true],
            vec![false, false, true],
            vec![false, false, true],
            vec![false, true, false],
            vec![true, false, false],
        ],
    );

    maps.insert(
        '?',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![false, false, false, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![false, false, false, false],
            vec![false, true, false, false],
        ],
    );

    maps.insert(
        '@',
        vec![
            vec![false, true, true, true, false],
            vec![true, false, false, false, true],
            vec![true, false, true, true, true],
            vec![true, false, true, true, true],
            vec![true, false, true, true, false],
            vec![true, false, false, false, false],
            vec![false, true, true, true, false],
        ],
    );

    // Uppercase letters
    maps.insert(
        'A',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'B',
        vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
        ],
    );

    maps.insert(
        'C',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        'D',
        vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
        ],
    );

    maps.insert(
        'E',
        vec![
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, true],
        ],
    );

    maps.insert(
        'F',
        vec![
            vec![true, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
    );

    maps.insert(
        'G',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, false],
            vec![true, false, true, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
        ],
    );

    maps.insert(
        'H',
        vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'I',
        vec![
            vec![true, true, true],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
    );

    maps.insert(
        'J',
        vec![
            vec![false, false, true, true],
            vec![false, false, false, true],
            vec![false, false, false, true],
            vec![false, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        'K',
        vec![
            vec![true, false, false, true],
            vec![true, false, true, false],
            vec![true, true, false, false],
            vec![true, false, false, false],
            vec![true, true, false, false],
            vec![true, false, true, false],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'L',
        vec![
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, true],
        ],
    );

    maps.insert(
        'M',
        vec![
            vec![true, false, false, false, true],
            vec![true, true, false, true, true],
            vec![true, false, true, false, true],
            vec![true, false, true, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
        ],
    );

    maps.insert(
        'N',
        vec![
            vec![true, false, false, true],
            vec![true, true, false, true],
            vec![true, true, false, true],
            vec![true, false, true, true],
            vec![true, false, true, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'O',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        'P',
        vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
        ],
    );

    maps.insert(
        'Q',
        vec![
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, true, true],
            vec![true, false, false, true],
            vec![false, true, true, true],
        ],
    );

    maps.insert(
        'R',
        vec![
            vec![true, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, true, true, false],
            vec![true, false, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'S',
        vec![
            vec![false, true, true, true],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![false, true, true, false],
            vec![false, false, false, true],
            vec![false, false, false, true],
            vec![true, true, true, false],
        ],
    );

    maps.insert(
        'T',
        vec![
            vec![true, true, true, true, true],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
            vec![false, false, true, false, false],
        ],
    );

    maps.insert(
        'U',
        vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
        ],
    );

    maps.insert(
        'V',
        vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
            vec![false, false, true, false],
        ],
    );

    maps.insert(
        'W',
        vec![
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, true, false, true],
            vec![true, false, true, false, true],
            vec![true, true, false, true, true],
            vec![true, false, false, false, true],
        ],
    );

    maps.insert(
        'X',
        vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
            vec![false, false, false, false],
            vec![false, true, true, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'Y',
        vec![
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![false, true, true, false],
            vec![false, false, true, false],
            vec![false, false, true, false],
            vec![false, false, true, false],
            vec![false, false, true, false],
        ],
    );

    maps.insert(
        'Z',
        vec![
            vec![true, true, true, true],
            vec![false, false, false, true],
            vec![false, false, true, false],
            vec![false, true, false, false],
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![true, true, true, true],
        ],
    );

    // Symbols
    maps.insert(
        '[',
        vec![
            vec![true, true],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![true, false],
            vec![true, true],
        ],
    );

    maps.insert(
        '\\',
        vec![
            vec![true, false, false, false],
            vec![true, false, false, false],
            vec![false, true, false, false],
            vec![false, true, false, false],
            vec![false, false, true, false],
            vec![false, false, true, false],
            vec![false, false, false, true],
        ],
    );

    maps.insert(
        ']',
        vec![
            vec![true, true],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![false, true],
            vec![true, true],
        ],
    );

    maps.insert(
        '^',
        vec![
            vec![false, true, false],
            vec![true, false, true],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
        ],
    );

    maps.insert(
        '_',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![false, false, false],
            vec![true, true, true],
        ],
    );

    maps.insert(
        '`',
        vec![
            vec![true, false],
            vec![false, true],
            vec![false, false],
            vec![false, false],
            vec![false, false],
            vec![false, false],
            vec![false, false],
        ],
    );

    // Lowercase letters
    maps.insert(
        'a',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, true, true],
            vec![false, false, true],
            vec![false, true, true],
            vec![true, false, true],
            vec![false, true, true],
        ],
    );

    maps.insert(
        'b',
        vec![
            vec![true, false, false],
            vec![true, false, false],
            vec![true, true, false],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, true, false],
        ],
    );

    maps.insert(
        'c',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, true, true],
            vec![true, false, false],
            vec![true, false, false],
            vec![true, false, false],
            vec![false, true, true],
        ],
    );

    maps.insert(
        'd',
        vec![
            vec![false, false, true],
            vec![false, false, true],
            vec![false, true, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![false, true, true],
        ],
    );

    maps.insert(
        'e',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, true, true],
            vec![true, false, true],
            vec![true, true, true],
            vec![true, false, false],
            vec![false, true, true],
        ],
    );

    maps.insert(
        'f',
        vec![
            vec![false, true, true],
            vec![true, false, false],
            vec![true, true, true],
            vec![true, false, false],
            vec![true, false, false],
            vec![true, false, false],
            vec![true, false, false],
        ],
    );

    maps.insert(
        'g',
        vec![
            vec![false, false, false],
            vec![false, true, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![false, true, true],
            vec![false, false, true],
            vec![false, true, false],
        ],
    );

    maps.insert(
        'h',
        vec![
            vec![true, false, false],
            vec![true, false, false],
            vec![true, true, false],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
        ],
    );

    maps.insert(
        'i',
        vec![
            vec![false, true, false],
            vec![false, false, false],
            vec![true, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
    );

    maps.insert(
        'j',
        vec![
            vec![false, false, true],
            vec![false, false, false],
            vec![false, true, true],
            vec![false, false, true],
            vec![false, false, true],
            vec![true, false, true],
            vec![false, true, false],
        ],
    );

    maps.insert(
        'k',
        vec![
            vec![true, false, false],
            vec![true, false, false],
            vec![true, false, true],
            vec![true, true, false],
            vec![true, true, false],
            vec![true, false, true],
            vec![true, false, true],
        ],
    );

    maps.insert(
        'l',
        vec![
            vec![true, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
        ],
    );

    maps.insert(
        'm',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![true, true, true, false],
            vec![true, false, true, true],
            vec![true, false, true, true],
            vec![true, false, true, true],
            vec![true, false, true, true],
        ],
    );

    maps.insert(
        'n',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![true, true, false],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
        ],
    );

    maps.insert(
        'o',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, true, false],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![false, true, false],
        ],
    );

    maps.insert(
        'p',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![true, true, false],
            vec![true, false, true],
            vec![true, true, false],
            vec![true, false, false],
            vec![true, false, false],
        ],
    );

    maps.insert(
        'q',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, true, true],
            vec![true, false, true],
            vec![false, true, true],
            vec![false, false, true],
            vec![false, false, true],
        ],
    );

    maps.insert(
        'r',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![true, false, true],
            vec![true, true, false],
            vec![true, false, false],
            vec![true, false, false],
            vec![true, false, false],
        ],
    );

    maps.insert(
        's',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![false, true, true],
            vec![true, false, false],
            vec![false, true, false],
            vec![false, false, true],
            vec![true, true, false],
        ],
    );

    maps.insert(
        't',
        vec![
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, true],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, true],
        ],
    );

    maps.insert(
        'u',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![false, true, true],
        ],
    );

    maps.insert(
        'v',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![true, false, true],
            vec![true, false, true],
            vec![true, false, true],
            vec![false, true, false],
            vec![false, true, false],
        ],
    );

    maps.insert(
        'w',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![true, false, false, true],
            vec![true, false, false, true],
            vec![true, false, true, true],
            vec![true, true, false, true],
            vec![true, false, false, true],
        ],
    );

    maps.insert(
        'x',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![true, false, true],
            vec![true, false, true],
            vec![false, true, false],
            vec![true, false, true],
            vec![true, false, true],
        ],
    );

    maps.insert(
        'y',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![true, false, true],
            vec![true, false, true],
            vec![false, true, true],
            vec![false, false, true],
            vec![false, true, false],
        ],
    );

    maps.insert(
        'z',
        vec![
            vec![false, false, false],
            vec![false, false, false],
            vec![true, true, true],
            vec![false, false, true],
            vec![false, true, false],
            vec![true, false, false],
            vec![true, true, true],
        ],
    );

    maps.insert(
        '{',
        vec![
            vec![false, true, true],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, false, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, true],
        ],
    );

    maps.insert(
        '|',
        vec![
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, true, false],
        ],
    );

    maps.insert(
        '}',
        vec![
            vec![true, true, false],
            vec![false, true, false],
            vec![false, true, false],
            vec![false, false, true],
            vec![false, true, false],
            vec![false, true, false],
            vec![true, true, false],
        ],
    );

    maps.insert(
        '~',
        vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![false, true, false, true],
            vec![true, false, true, false],
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![false, false, false, false],
        ],
    );

    // Add a fallback for unknown characters
    maps.insert(
        '�',
        vec![
            vec![true, true, true, true, true],
            vec![true, false, false, false, true],
            vec![true, false, true, false, true],
            vec![true, false, true, false, true],
            vec![true, false, true, false, true],
            vec![true, false, false, false, true],
            vec![true, true, true, true, true],
        ],
    );

    maps
}
