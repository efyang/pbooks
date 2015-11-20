use hyper::client::*;
use hyper::header::{ContentLength, ContentType};
use hyper::mime::Mime;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::hash::{Hash, SipHasher, Hasher};
use std::iter;
use time::precise_time_s;
use term_painter::ToStyle;
use term_painter::Color::*;

//TODO: make downloaded files go in directory directly related to the executable
//use time to get kb/s remove any raw unwrap()s as possible
//sigint for thread

#[cfg(unix)]
const FILE_SEP: &'static str = "/";

#[cfg(windows)]
const FILE_SEP: &'static str = "\\";

pub fn download_pdf_to_default_url_file(url: &str) -> Result<(), String> {
    let filename;
    match get_url_file(url) {
        Some(u) => filename = u.to_string(),
        //fallback is just hash of url + filetype
        None => {
            let mut s = SipHasher::new();
            url.hash(&mut s);
            filename = s.finish().to_string() + ".pdf";
        },
    }
    download_pdf_to_file(url, &filename)
}

pub fn download_pdf_to_file(url: &str, outputfile: &str) -> Result<(), String> {
    let mut outfile = BufWriter::new(File::create(outputfile)
                                     .expect(&format!("Failed to create file {}", outputfile)));
    let client = Client::new();
    let stream = client.get(url).send().unwrap();
    if !is_pdf(&stream) {
        return Err("Not a valid PDF file url".to_string());
    }
    let contentlen = get_content_length(&stream).unwrap_or_else(|| {
        println!("Warning: Failed to get file download size");
        0
    });
    let contentstr = convert_to_apt_unit(contentlen);
    println!(" {} {} from url: \"{}\" to \"{}\"",
             BrightGreen.bold().paint("Downloading"), contentstr, url, outputfile);
    let bytes_read = Arc::new(Mutex::new(0));
    let stop_printing = Arc::new(Mutex::new(false));

    {
        let bytes_read = bytes_read.clone();
        let stop_printing = stop_printing.clone();
        let outputfile = outputfile.to_string();
        thread::spawn(move || {
            let start_time = precise_time_s();
            loop {
                thread::sleep(Duration::from_millis(0));
                let bytes_read = bytes_read.lock().unwrap();
                print_dl_status(*bytes_read, contentlen, &contentstr);
                if *stop_printing.lock().unwrap() {
                    print_dl_status(*bytes_read, contentlen, &contentstr);
                   println!("\n   {} Download of file \"{}\"in {:.5} seconds",
                             BrightGreen.bold().paint("Completed"), outputfile,
                             round_to_places(precise_time_s() - start_time, 5));
                    break;
                }
            }
        });
    }

    for byte in stream.bytes() {
        let mut bytes_read = bytes_read.lock().unwrap();
        *bytes_read += 1;
        outfile.write(&[byte.unwrap()]).unwrap();
    }

    let mut stop_printing = stop_printing.lock().unwrap();
    *stop_printing = true;
    return Ok(());
}



fn get_content_length(r: &Response) -> Option<u64> {
    match r.headers.get::<ContentLength>() {
        Some(c) => {
            let ContentLength(contentlen) = *c;
            Some(contentlen)
        },
        None => None,
    }
}

fn is_pdf(r: &Response) -> bool {
    match r.headers.get::<ContentType>() {
        Some(c) => {
            let ContentType(ref contenttype) = *c;
            let pdf: Mime = "application/pdf".parse().unwrap();
            if contenttype == &pdf {
                true
            } else {
                false
            }
        },
        None => false,
    }
}

//maybe use io::stdout() to prevent the weird cursor?
fn print_dl_status(done: u64, total: u64, totalstr: &str) {
    let dl = BrightGreen.bold().paint(" Downloaded");
    let aptconversion = convert_to_apt_unit(done);
    if total == 0 {
        print!("\r {dl} {dledbytes} of unknown | unknown% complete          ",
               dl = dl, dledbytes = aptconversion);
    } else {
        let percentdone: f64 = round_to_places(((done as f64/total as f64) * 100f64), 2);
        print!("\r {dl} {dledbytes} of {length} | {percent:.2}% complete          ",
            dl = dl, dledbytes = aptconversion, length = totalstr, percent = percentdone);
    }
}

fn get_url_file(url: &str) -> Option<&str> {
    url.split('/').last()
}

fn convert_to_apt_unit(bytelength: u64) -> String {
    let unit;
    let divisor;
    if bytelength < 1024 {
        divisor = 1;
        unit = "B";
    } else if bytelength >= 1024 && bytelength < 1048576 {
        divisor = 1024;
        unit = "KiB";
    } else if bytelength >= 1048576 && bytelength < 1073741800 {
        divisor = 1048576;
        unit = "MiB";
    } else {
        divisor = 1073741800;
        unit = "GiB";
    }
    format!("{:.2} {}", round_to_places(bytelength as f64/divisor as f64, 2), unit)
}

const ZERO: &'static str = "0";

//places refers to places after decimal point
fn round_to_places(n: f64, places: usize) -> f64 {
    let div = ("1".to_string() + &ZERO.to_string().repeat(places)).parse::<f64>().unwrap();
    (n * div).round() / div
}

trait Repeatable {
    fn repeat(&self, times: usize) -> String;
}

impl Repeatable for String {
    fn repeat(&self, times: usize) -> String {
        iter::repeat(self).take(times).map(|s| s.clone()).collect::<String>()
    }
}
