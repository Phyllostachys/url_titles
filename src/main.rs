use std::io::prelude::*;
use std::sync::Arc;

extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate reqwest;

extern crate select;
use select::document::Document;
use select::predicate::Name;

error_chain! {
    foreign_links {
        ReqError(reqwest::Error);
        IoError(std::io::Error);
    }
}

struct UrlTitle {
    url: std::string::String,
    title: std::string::String,
}

fn run() -> Result<()> {
    env_logger::init();

    let path;
    if let Some(arg1) = std::env::args().nth(1) {
        path = arg1;
    } else {
        std::process::exit(-1);
    }

    let file = std::fs::File::open(path)?;
    let buf = std::io::BufReader::new(file);
    let mut urls : Vec<UrlTitle> = Vec::new();
    for line in buf.lines() {
        match line {
            Ok(l) => urls.push(UrlTitle {
                url: l,
                title: String::new(),
            }),
            Err(e) => println!("Error on getting line... {}", e),
        }
    }
    let mut url_titles: Arc<Vec<UrlTitle>> = Arc::new(urls);

    let mut threads = Vec::new();
    for i in 0..url_titles.len() {
        let ut = Arc::clone(&url_titles);
        threads.push(std::thread::spawn(move || {
            let mut res;
            //unsafe {
                res = reqwest::get(ut[i].url.as_str()).unwrap();
            //}
            let content = res.text().unwrap();
            let document = Document::from(content.as_str());
            let node = document
                .find(Name("title"))
                .nth(0)
                .expect("Node \"title\" not found...");
            unsafe {
                (*(ut.as_ptr() as *mut UrlTitle).offset(i as isize)).title = node.text();
            }
        }));
        std::thread::sleep(std::time::Duration::from_millis(750));
    }

    for thread in threads {
        thread.join();
    }

    for url_title in url_titles.iter() {
        println!("{} --- {}", url_title.url, url_title.title);
    }

    Ok(())
}

quick_main!(run);
