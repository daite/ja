use std::io::Write;
use select::document::Document;
use select::predicate::{Name};
use std::error::Error;
use std::fs::File;
use reqwest;
use tokio;
use std::{env, fs};

const BASE_URL: &str = "https://www.jav321.com/video/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut tasks = vec![];
    let start: i32 = args[2].parse().unwrap();
    let end: i32 = args[3].parse().unwrap();
    for num in start..end {
        let item_name = args[1].to_owned();
        tasks.push(tokio::spawn(async move {
            let _ = execute(&item_name, num).await;
        }));
    }
    for task in tasks {
        task.await?
    }
    Ok(())
}

async fn execute(item_name: &str, num: i32) -> Result<(), Box<dyn Error>> {
    let mut tasks = vec![];
    let item = format!("{}-{}", item_name, num);
    let url = format!("{}{}", BASE_URL, item);
    let links = get_links(&url).await?;
    // path
    let path = env::current_dir().unwrap();
    let p = path.to_str().unwrap();
    let base_path = format!("{}/{}", p, "images");
    // tasks
    if links.len() > 1 {
        println!("{:#?}", links);
        for link in links {
            let final_path = format!("{}/{}", base_path, item);
            tasks.push(tokio::spawn(async move {
                fs::create_dir_all(&final_path).unwrap();
                let _ = save_img_file(&final_path, &link).await;
            }));
        }
        for task in tasks {
            task.await?
        }
    }
    Ok(())
}

async fn get_doc(url: &str) -> Result<Document, Box<dyn Error>>{
    let contents = reqwest::get(url)
        .await?
        .text()
        .await?;
    let doc = Document::from(&contents[..]);
    Ok(doc)
}

async fn save_img_file(final_path: &str, url: &str) -> Result<(), Box<dyn Error>> {
    println!("{:?}", url);
    let file_name = url.split("/").collect::<Vec<&str>>();
    let file_name = file_name[file_name.len()-1];
    let bytes = reqwest::get(url)
        .await?
        .bytes()
        .await?;
    let save_dir_path = format!("{}/{}", final_path, file_name);
    println!("[*] Saving to {}", save_dir_path);
    let mut file = File::create(save_dir_path)?;
    file.write_all(&bytes)?;
    Ok(())
}

async fn get_links(url: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut links = vec![];
    let doc = get_doc(url).await?;
    for node in doc.find(Name("p")) {
        for n in node.first_child() {
            if let Some(img) = n.children().next() {
                let link = img.attr("src").unwrap();
                links.push(link.to_string());
            } 
        }
    }
    Ok(links)
}
