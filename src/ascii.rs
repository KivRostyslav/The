use opencv::{core::MatTraitConst, prelude::Mat, core::Vec3b};

use crate::terminal::StringInfo;

pub struct AsciiConverter {
    chars: Vec<char>,
    step: u32,
}

// ASCII-127 Only
pub const CHARS1: &str = r##" .:-=+*#%@"##; // 10 chars
pub const CHARS2: &str = r##" .'`^",:;Il!i~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"##; // 67 chars
pub const CHARS3: &str = r##" `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@"##; // 92 chars

// ASCII-255
pub const SOLID: &str = r#"█"#; // 1 Solid block
pub const DOTTED: &str = r#"⣿"#; // 1 dotted block
pub const GRADIENT: &str = r#" ░▒▓█"#; // 5 chars
pub const BLACKWHITE: &str = r#" █"#; // 2 chars
pub const BW_DOTTED: &str = r#" ⣿"#; // 2 dotted block
pub const BRAILLE: &str = r#" ··⣀⣀⣤⣤⣤⣀⡀⢀⠠⠔⠒⠑⠊⠉⠁"#; // 16 chars (braille-based)
pub const NO: &str = r#"01234⣀5678"#;

pub const PROGMRAM: &str = r#"
__inline void write(__global uchar* output, __global uchar* input, uint output_start_index, uint input_start_index, uint len) {
    for (uint i = 0; i < len; i++) {
        output[output_start_index + i] = input[input_start_index + i];
    }
}

__kernel void calculate(__global uchar* frame, __global uchar* chars, uint char_len, uint grayscale, uint step, __global uchar* out) {
    int index = get_global_id(0);
    int brightness = frame[index];

    if (!grayscale) {
        brightness = (frame[index * 3] + frame[index * 3 + 1] + frame[index * 3 + 2]) / 3;
    }

    int char_index = brightness / step;
    write(out, chars, index * char_len, char_index * char_len, char_len);
}
"#;

impl AsciiConverter {
    pub fn new(string: &String) -> Self {
        let step = (255.0 / (string.chars().count() as f32)).ceil() as u32;
        
        Self {
            step,
            chars: string.chars().collect::<Vec<char>>().to_owned(),
        }
    }

    pub fn convert(&self, frame: &Mat, grayscale: bool) -> StringInfo {
        let frame_size = frame.size().unwrap();
        let mut string = String::with_capacity((frame_size.width * frame_size.height) as usize);
        let mut rgb = Vec::new();

        for row in 0..frame.rows() {
            string.push('\n');
            for column in 0..frame.cols() {
                let brightness: u32;
                if grayscale {
                    brightness = (*frame.at_2d::<u8>(row, column).unwrap()) as u32;
                }
                else {
                    let pixel = frame.at_2d::<Vec3b>(row, column).unwrap();
                    for color in pixel.iter() {
                        rgb.push(*color);
                    }
                    
                    brightness = pixel.iter().map(|&i| i as u32).sum::<u32>() / 3;
                }
                let index = ((brightness as f32) / self.step as f32).floor() - 1.0;
                string.push(self.chars[index as usize]); }
        }

        StringInfo {
            string: string.as_bytes().to_vec(),
            char_len: 0,
            rgb,
        }
    }
}
