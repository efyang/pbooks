extern crate hyper;
extern crate time;
extern crate term_painter;
extern crate terminal_size;
use term_painter::ToStyle;
use term_painter::Color::*;

mod download;
use download::download_pdf_to_default_url_file;

fn main() {
    match download_pdf_to_default_url_file("https://github.com/algorhythms/Algo-Quicksheet/releases/download/0.0.4/algo-quicksheet.pdf") {
        Ok(_) => {},
        Err(e) => println!("{}{}", BrightRed.bold().paint("Error: "), 
                           Red.bold().paint(e)),
    }
}
