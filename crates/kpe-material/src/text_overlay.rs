pub struct TextOverlayEngine;

impl TextOverlayEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn overlay_text(
        &self,
        pixels: &mut [u8],
        width: u32,
        height: u32,
        text: &str,
        x: u32,
        y: u32,
        font_size: u32,
    ) {
        if text.is_empty() {
            return;
        }

        let char_width = font_size / 2;
        let char_height = font_size;

        for (ci, c) in text.chars().enumerate() {
            let cx = x + ci as u32 * char_width;
            let cy = y;

            for py in cy..cy + char_height {
                for px in cx..cx + char_width {
                    if px < width && py < height {
                        let idx = ((py * width + px) * 4) as usize;
                        if idx + 3 < pixels.len() {
                            let pattern = self.simple_char_pattern(c, ci, px - cx, py - cy, char_width, char_height);
                            if pattern {
                                pixels[idx] = 0;
                                pixels[idx + 1] = 0;
                                pixels[idx + 2] = 0;
                                pixels[idx + 3] = 255;
                            }
                        }
                    }
                }
            }
        }
    }

    fn simple_char_pattern(
        &self,
        _c: char,
        _index: usize,
        px: u32,
        py: u32,
        _char_width: u32,
        char_height: u32,
    ) -> bool {
        if px == 0 || px == _char_width - 1 || py == 0 || py == char_height - 1 {
            return true;
        }
        false
    }
}

impl Default for TextOverlayEngine {
    fn default() -> Self {
        Self::new()
    }
}
