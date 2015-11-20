extern crate hyper;
extern crate time;
extern crate term_painter;

mod download;
use download::download_pdf_to_default_url_file;

fn main() {
    match download_pdf_to_default_url_file("http://www.jjj.de/fxt/fxtbook.pdf") {
        Ok(_) => {},
        Err(e) => println!("{}", e),
    }
}
