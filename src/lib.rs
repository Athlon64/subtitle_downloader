extern crate crypto;
extern crate futures;
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;
extern crate url;

use futures::{Future, Stream};
use tokio_core::reactor::Core;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::error::Error;

use crypto::digest::Digest;
use crypto::md5::Md5;
use hyper::{Client, Method, Request};
use hyper::header::{ContentLength, ContentType};

use url::*;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct SubInfo {
    Desc: String,
    Delay: i32,
    Files: Vec<Fileinfo>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct Fileinfo {
    Ext: String,
    Link: String,
}

pub fn down_sub(file_name: String) -> Result<(), Box<Error>> {
    let file_md5 = get_file_md5(&file_name)?;

    let query = vec![("filehash", file_md5.as_str()), ("format", "json")];
    let body = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(query.iter())
        .finish();

    let mut core = Core::new()?;
    let client = Client::new(&core.handle());
    let uri = "http://shooter.cn/api/subapi.php".parse()?;
    let mut req = Request::new(Method::Post, uri);
    req.headers_mut().set(ContentType::form_url_encoded());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.set_body(body);
    let post = client.request(req).and_then(|res| res.body().concat2());
    let res = core.run(post)?.to_vec();

    if res.len() < 10 {
        println!("抱歉，未找到字幕。");
    } else {
        down_sub_file(&file_name, res)?;
    }

    Ok(())
}

fn get_file_md5(file_name: &String) -> Result<String, Box<Error>> {
    let file_size = std::fs::metadata(&file_name)?.len();
    let mut file = File::open(&file_name)?;
    let offsets = vec![4096, file_size / 3 * 2, file_size / 3, file_size - 8192];
    let mut buffer: [u8; 4096] = [0; 4096];
    let mut file_md5 = String::new();
    for offset in offsets {
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut buffer)?;
        file_md5.push_str(&get_chunk_md5(&buffer));
        file_md5.push_str(";");
    }
    file_md5.pop();

    Ok(file_md5)
}

fn get_chunk_md5(chunk: &[u8]) -> String {
    let mut chunk_md5 = Md5::new();
    chunk_md5.input(&chunk);
    chunk_md5.result_str()
}

fn down_sub_file(file_name: &String, sub_res: Vec<u8>) -> Result<(), Box<Error>> {
    let sub_info: Vec<SubInfo> = serde_json::from_slice(&sub_res)?;
    for sub_file in &sub_info[0].Files {
        let mut core = Core::new()?;
        let client = Client::new(&core.handle());

        let work = client
            .get((&sub_file.Link.replace("https", "http")).parse()?)
            .and_then(|res| res.body().concat2());

        let mut f = File::create(file_name.clone() + "." + &sub_file.Ext)?;
        f.write_all(&core.run(work)?.to_vec())?;
    }

    println!("下载完成！");
    Ok(())
}
