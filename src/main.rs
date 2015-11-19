extern crate hyper;
mod download;
use download::download_pdf_to_default_url_file;
fn main() {
    download_pdf_to_default_url_file("http://www.jjj.de/fxt/fxtbook.pdf");
}
