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

//TODO:
//make downloaded files go in directory directly related to the executable
//use chrono to get kb/s

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
    let mut outfile = BufWriter::new(File::create(outputfile).expect(&format!("Failed to create file {}", outputfile)));
    let client = Client::new();
    println!(" Downloading from url: {} to file {}", url, outputfile);
    let stream = client.get(url).send().unwrap();
    if !is_pdf(&stream) {
        return Err("Not a valid PDF file url".to_string());
    }
    let contentlen = get_content_length(&stream).unwrap_or_else(|| {
        println!("Warning: Failed to get download size");
        0
    });
    let kbcontentlen = contentlen/1024;
    let bytes_read = Arc::new(Mutex::new(0));
    let stop_printing = Arc::new(Mutex::new(false));
    {
        let bytes_read = bytes_read.clone();
        let stop_printing = stop_printing.clone();
        let outputfile = outputfile.to_string();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(0));
                let bytes_read = bytes_read.lock().unwrap();
                print_dl_status(*bytes_read, contentlen, kbcontentlen);
                if *stop_printing.lock().unwrap() {
                    print_dl_status(*bytes_read, contentlen, kbcontentlen);
                    println!("\n Completed download of file: {}", outputfile);
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

fn print_dl_status(done: u64, total: u64, kbtotal: u64) {
    if total == 0 {
        print!("\r Downloaded: {dledbytes} kB of unknown kB | unknown% complete", dledbytes = done/1024);
    } else {
        let percentdone: u64 = ((done as f64/total as f64) * 100f64) as u64;
        print!("\r Downloaded: {dledbytes} kB of {kblength} kB | {percent}% complete", 
            dledbytes = done/1024, 
            kblength = kbtotal,
            percent = percentdone);
    }
}

fn get_url_file(url: &str) -> Option<&str> {
    url.split('/').last()
}    
