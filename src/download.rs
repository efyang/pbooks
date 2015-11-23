use hyper::client::*;
use hyper::header::ContentLength;
use std::fs::{File, rename};
use std::io::prelude::*;
use std::io::BufWriter;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::hash::{Hash, SipHasher, Hasher};
use std::iter;
use time::precise_time_s;
use terminal_size::{Width, Height, terminal_size};
use std::io::stdout;
use std::sync::mpsc::{channel, Receiver};

// TODO: make downloaded files go in directory directly related to the executable
// use time to get kb/s remove any raw unwrap()s as possible
// sigint for thread

#[cfg(unix)]
const FILE_SEP: &'static str = "/";

#[cfg(windows)]
const FILE_SEP: &'static str = "\\";

pub fn download_pdf_to_default_url_file(url: &str) -> Result<(), String> {
    let filename;
    match get_url_file(url) {
        Some(u) => filename = u.to_string(),
        // fallback is just hash of url + filetype
        None => {
            let mut s = SipHasher::new();
            url.hash(&mut s);
            filename = s.finish().to_string() + ".pdf";
        }
    }
    download_pdf_to_file(url, &filename)
}

pub fn download_pdf_to_file(url: &str, outputfile: &str) -> Result<(), String> {
    let mut outfile = BufWriter::new(File::create(format!("{}.tmp", outputfile))
                                         .expect(&format!("Failed to create file {}", outputfile)));
    let client = Client::new();
    let stream = client.get(url).send().unwrap();
    if !is_pdf(url) {
        return Err("Not a valid PDF file url".to_string());
    }
    let contentlen = get_content_length(&stream).unwrap_or_else(|| {
        println!("Warning: Failed to get file download size");
        0
    });
    let contentstr = convert_to_apt_unit(contentlen);
    let bytes_read = Arc::new(Mutex::new(0));
    let stop_printing = Arc::new(Mutex::new(false));

    {
        let bytes_read = bytes_read.clone();
        let stop_printing = stop_printing.clone();
        let outputfile = outputfile.to_string();
        thread::spawn(move || {
            println!("");
            let start_time = precise_time_s();
            loop {
                thread::sleep(Duration::from_millis(0));
                let bytes_read = bytes_read.lock().unwrap();
                print_dl_status(&outputfile,
                                *bytes_read,
                                contentlen,
                                &contentstr,
                                start_time);
                if *stop_printing.lock().unwrap() {
                    print_dl_status(&outputfile,
                                    *bytes_read,
                                    contentlen,
                                    &contentstr,
                                    start_time);
                    print_completed_dl(start_time, outputfile);
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
    rename(format!("{}.tmp", outputfile), outputfile).expect("Failed to rename file");
    return Ok(());
}

struct Download<'a> {
    pub url: &'a str,
    pub outfile: &'a str,
    pub enabled: bool,
}
// parallel downloads
// result is either nothing or vec of failed urls
// each thread sends the update message to the main print thread
// print thread should sort by given id -> append id at head of info?
// use channels for receiving the messages
// hashmap -> dlID : current message
// on recv update
#[allow(unused_variables)]
fn parallel_download_pdfs(urls: Vec<&str>) -> Result<(), Vec<&str>> {
    unimplemented!();
}

// String fail message
#[allow(unused_variables)]
fn parallel_download_single(url: &str,
                            outputfile: &str,
                            messagerecv: Receiver<String>)
                            -> Result<(), String> {
    let mut outfile = BufWriter::new(File::create(format!("{}.tmp", outputfile))
                                         .expect(&format!("Failed to create file {}", outputfile)));
    let client = Client::new();
    let stream = client.get(url).send().unwrap();
    if !is_pdf(url) {
        return Err("Not a valid PDF file url".to_string());
    }
    let contentlen = get_content_length(&stream).unwrap_or_else(|| {
        println!("Warning: Failed to get file download size");
        0
    });
    let contentstr = convert_to_apt_unit(contentlen);
    let bytes_read = Arc::new(Mutex::new(0));
    let stop_printing = Arc::new(Mutex::new(false));

    {
        let bytes_read = bytes_read.clone();
        let stop_printing = stop_printing.clone();
        let outputfile = outputfile.to_string();
        thread::spawn(move || {
            let start_time = precise_time_s();
            println!("");
            loop {
                thread::sleep(Duration::from_millis(0));
                let bytes_read = bytes_read.lock().unwrap();
                print_dl_status(&outputfile,
                                *bytes_read,
                                contentlen,
                                &contentstr,
                                start_time);
                if *stop_printing.lock().unwrap() {
                    print_dl_status(&outputfile,
                                    *bytes_read,
                                    contentlen,
                                    &contentstr,
                                    start_time);
                    print_completed_dl(start_time, outputfile);
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
    rename(format!("{}.tmp", outputfile), outputfile).expect("Failed to rename file");
    return Ok(());

    unimplemented!();
}

fn get_content_length(r: &Response) -> Option<u64> {
    match r.headers.get::<ContentLength>() {
        Some(c) => {
            let ContentLength(contentlen) = *c;
            Some(contentlen)
        }
        None => None,
    }
}

fn is_pdf(url: &str) -> bool {
    url.to_lowercase().contains(".pdf")
}

fn print_completed_dl(start_time: f64, filename: String) {
    println!(" Completed download of file \"{}\" in {:.5} seconds",
             filename,
             round_to_places(precise_time_s() - start_time, 5));
}

const PBAR_FORMAT: &'static str = "[=> ]";
const PBAR_LENGTH: usize = 35;

fn print_dl_status(filename: &str, done: u64, total: u64, totalstr: &str, start_time: f64) {
    let dledbytes = convert_to_apt_unit(done).autopad(7);
    let pbar;
    let length;
    let strpercent;
    if total == 0 {
        pbar = make_progress_bar(PBAR_FORMAT, PBAR_LENGTH, 0.0);
        length = "N/A";
        strpercent = "N/A".to_string();
    } else {
        let percentdone: f64 = round_to_places(((done as f64 / total as f64) * 100f64), 2);
        strpercent = format!("{:.2}", percentdone).to_string().autopad(6);
        pbar = make_progress_bar(PBAR_FORMAT, PBAR_LENGTH, percentdone);
        length = totalstr;
    }
    let msg = format!("{}/{}", dledbytes, length);
    let vmsg = format!("{pbar} {percent}%", percent = strpercent, pbar = pbar);
    clear_lines(1);
    if let Some((Width(w), Height(_))) = terminal_size() {
        println!(" {} {} {} ",
               filename,
               msg,
               vmsg.pad((w as usize - (filename.len() + msg.len() + 5) - (PBAR_LENGTH + 8))));
    } else {
        println!(" {} {} {} ", filename, msg, vmsg);
    }
    let stdout = stdout();
    let mut handle = stdout.lock();
    handle.flush().expect("Failed to flush stdout");
}

fn clear_lines(lines: usize) {
    for _ in 0..lines {
        //println!("");
        print!("\x1b[1A");
        print!("\x1b[K");
    }
}

// formatting is in format "<start><filled><filledhead><empty><end>"
// example: "[=>-]"
fn make_progress_bar(formatting: &str, barlength: usize, percent: f64) -> String {
    let mut formatiter = formatting.chars();
    let startchar = formatiter.next().unwrap();
    let fillchar = formatiter.next().unwrap();
    let headchar = formatiter.next().unwrap();
    let emptychar = formatiter.next().unwrap();
    let endchar = formatiter.next().unwrap();
    let proglength = barlength - 2;
    let headidx: usize = (proglength as f64 * (percent as f64 / 100.0)) as usize;
    let bar: String = format!("{}{}{}{}{}",
                              startchar,
                              fillchar.to_string().repeat(headidx),
                              headchar,
                              emptychar.to_string().repeat(proglength - headidx),
                              endchar);
    bar
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
        unit = "K";
    } else if bytelength >= 1048576 && bytelength < 1073741800 {
        divisor = 1048576;
        unit = "M";
    } else {
        divisor = 1073741800;
        unit = "G";
    }
    format!("{:.1}{}",
            round_to_places(bytelength as f64 / divisor as f64, 1),
            unit)
}

const ZERO: &'static str = "0";

// places refers to places after decimal point
fn round_to_places(n: f64, places: usize) -> f64 {
    let div = ("1".to_string() + &ZERO.to_string().repeat(places)).parse::<f64>().unwrap();
    (n * div).round() / div
}

trait PrettyPrint {
    fn repeat(&self, times: usize) -> String;
    fn pad(&self, amnt: usize) -> String;
    fn autopad(&self, goalchars: usize) -> String;
}

impl PrettyPrint for str {
    fn repeat(&self, times: usize) -> String {
        iter::repeat(self).take(times).map(|s| s.clone()).collect::<String>()
    }
    fn pad(&self, amnt: usize) -> String {
        format!("{}{}", " ".repeat(amnt), self)
    }
    fn autopad(&self, goalchars: usize) -> String {
        let cursize = self.len();
        if cursize >= goalchars {
            self.to_string()
        } else {
            self.pad(goalchars - cursize)
        }
    }
}
