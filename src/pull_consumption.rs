use reqwest::Client;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::PathBuf;

use crate::utils::{logger, LoggerSeverity};

const TEMP_FOLDER: &str = "./temp";
const DATA_FOLDER: &str = "./temp/data";
const DATA_FILE_NAME: &str = "data.txt";
const CONSUMPTION_FILE_NAME: &str = "consumption.txt";
const RESIDENCE_LIST: [&str; 11] = [
    "Farmer",
    "Worker",
    "Artisan",
    "Engineer",
    "Investor",
    "Scholar",
    "Jornalero",
    "Explorer",
    "Technician",
    "Shepherd",
    "Elder",
];

fn get_residence_set() -> HashSet<String> {
    RESIDENCE_LIST.iter().map(|r| r.to_string()).collect()
}

async fn load_data() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    create_temp_folder(false)?;
    create_data_folder(false)?;
    let data_file = get_data_file_path();
    let residence_set = get_residence_set();

    let mut tables = Vec::new();
    if !data_file.exists() {
        for residence in residence_set.iter() {
            logger(
                LoggerSeverity::Info,
                &format!("Downloading data for {}", residence),
            );

            let url = format!("https://anno1800.fandom.com/wiki/{}_Residence", residence);

            let resident_content = get_content_from_url(&url).await;

            tables.extend(resident_content);
        }
    } else {
        logger(LoggerSeverity::Info, "Loading data from file");
        tables.extend(get_content_from_file());
    }

    create_file(&data_file, tables.clone().join("|").as_str())?;

    Ok(tables)
}

pub async fn pull_consumption_data() -> Result<(), Box<dyn std::error::Error>> {
    let residence_set = get_residence_set();

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut rows_loaded = 0;

    let tables = load_data().await?;

    logger(LoggerSeverity::Info, "Finding Tables");

    // find each residence table
    let residence_tables = tables.iter().filter(|t| {
        let tr = select_by_regex(&t, r#"<tr.*?>(.*?)</tr>"#, false);
        let title = select_by_regex(&tr[0], r#"<th.*?>(.*?)</th>"#, true)[0]
            .clone()
            .trim_end_matches("s")
            .to_string();

        residence_set.contains(&title)
    });

    rows.extend(residence_tables.map(|t| {
        let tr = select_by_regex(&t, r#"<tr.*?>(.*?)</tr>"#, false);

        if tr.len() == 0 {
            return Vec::new();
        }

        let title = select_by_regex(&tr[0], r#"<th.*?>(.*?)</th>"#, true)[0]
            .clone()
            .trim_end_matches("s")
            .to_string();

        if !residence_set.contains(&title) {
            return Vec::new();
        }

        let mut rows = Vec::new();
        for row in tr {
            let th = select_by_regex(&row, r#"<th.*?>(.*?)</th>"#, true);
            let td = select_by_regex(&row, r#"<td.*?>(.*?)</td>"#, true);

            if th.len() == 0 {
                continue;
            }

            rows_loaded += 1;

            // join th and td
            let row_joined = th.iter().chain(td.iter()).map(|s| s.to_string()).collect();
            rows.push(row_joined);
        }

        logger(LoggerSeverity::Info, &format!("{} parsed", title));

        rows
    }));

    let temp_path = get_temp_path().join(CONSUMPTION_FILE_NAME);
    // save vector of rows to text file
    let mut file = File::create(temp_path)?;
    for row in rows {
        println!("{:?}", row);
        file.write_all(format!("{:?}\n", row).as_bytes())?;
    }

    logger(
        LoggerSeverity::Info,
        &format!("{} rows loaded", rows_loaded),
    );

    Ok(())
}

fn select_by_regex(html: &str, regex: &str, is_remove_html_tags: bool) -> Vec<String> {
    let re = regex::Regex::new(regex).unwrap();
    let value = re
        .captures_iter(&html)
        .map(|cap| cap[1].to_string())
        .collect::<Vec<String>>();

    // if is remove html tags then remove all tags from the string, anything between < and >
    if is_remove_html_tags {
        let re = regex::Regex::new(r#"<.*?>"#).unwrap();

        let mut value = value;
        for i in 0..value.len() {
            value[i] = re.replace_all(&value[i], "").to_string();
        }
        // trim
        for i in 0..value.len() {
            value[i] = value[i].trim().to_string();
        }

        return value;
    }

    value
}

fn create_temp_folder(clean: bool) -> std::io::Result<PathBuf> {
    let file_path = get_temp_path();
    if file_path.exists() {
        if clean {
            std::fs::remove_dir_all(&file_path)?;
        }
    } else {
        std::fs::create_dir(&file_path)?;
    }

    Ok(file_path)
}

fn create_data_folder(clean: bool) -> std::io::Result<PathBuf> {
    let file_path = get_temp_data_path();
    if file_path.exists() {
        if clean {
            std::fs::remove_dir_all(&file_path)?;
        }
    } else {
        std::fs::create_dir(&file_path)?;
    }

    Ok(file_path)
}

fn create_file(file_name: &PathBuf, content: &str) -> std::io::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn get_content_from_file() -> Vec<String> {
    let file_path = get_data_file_path();

    fs::read_to_string(&file_path)
        .unwrap()
        .split("|")
        .map(|s| s.to_string())
        .collect()
}

fn get_temp_path() -> PathBuf {
    let mut file_path = PathBuf::new();

    file_path.push(TEMP_FOLDER);

    file_path
}

fn get_temp_data_path() -> PathBuf {
    let mut file_path = PathBuf::new();

    file_path.push(DATA_FOLDER);

    file_path
}

fn get_data_file_path() -> PathBuf {
    let mut file_path = PathBuf::new();

    file_path.push(DATA_FOLDER);
    file_path.push(DATA_FILE_NAME);

    file_path
}

async fn get_content_from_url(url: &str) -> Vec<String> {
    let client = Client::new();
    let res = client.get(url).send().await.unwrap();
    let raw_html = res.text().await.unwrap();
    // remove \n
    let html = raw_html.replace("\n", "");
    let tables = select_by_regex(&html, r#"<table.*?>(.*?)</table>"#, false);

    tables
}
