extern crate hyper;
extern crate time;
extern crate term_painter;
use term_painter::ToStyle;
use term_painter::Color::*;

mod download;
use download::download_pdf_to_default_url_file;

fn main() {
    match download_pdf_to_default_url_file("http://www.jjj.de/fxt/fxtbook.pdf") {
        Ok(_) => {},
        Err(e) => println!("{}{}", BrightRed.bold().paint("Error: "), 
                           Red.bold().paint(e)),
    }
}
