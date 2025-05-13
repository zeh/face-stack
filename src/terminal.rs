// From https://github.com/zeh/random-art-generator/blob/main/src/generator/utils/terminal.rs

pub fn cursor_up() {
	print!("\u{1b}[1;A");
}

pub fn erase_line_to_end() {
	print!("\u{1b}[0;K");
}
