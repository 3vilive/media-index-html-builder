use std::{fs, path::PathBuf, io::Write};

use anyhow::{{Result}};
use clap::{Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    base_dir: String,
}



fn main() -> Result<()> {
    // get args
    let args = Args::parse();
    let base_dir = &args.base_dir;

    // get sub dirs
    let sub_dirs = get_sub_dirs(base_dir)?; 
    println!("get dirs: {:?}", sub_dirs);

    // get media entries
    let mut media_entries = vec![];
    for sub_dir in sub_dirs.iter() {
        media_entries.push(get_media_entry(sub_dir)?);
    }
    media_entries.sort_by(|a, b| a.dir_path.cmp(&b.dir_path));
    println!("get media entries: {:?}", media_entries);

    build_index_html(media_entries, base_dir)
}

fn get_sub_dirs(base_dir: &str) -> Result<Vec<PathBuf>> {
    let mut sub_dirs = vec![];
    for entry in fs::read_dir(base_dir)? {
        let path = entry?.path();
        let file_name = path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        
        if !path.is_dir() || file_name.is_empty() || file_name.starts_with(".") {
            continue
        }

        sub_dirs.push(path.clone());
    }

    Ok(sub_dirs)
}

#[derive(Debug)]
struct MediaEntry {
    dir_path: PathBuf,
    images: Vec<PathBuf>,
    videos: Vec<PathBuf>,
}

fn get_media_entry(dir: &PathBuf) -> Result<MediaEntry> {
    let mut images = vec![];
    let mut videos = vec![];

    for entry in fs::read_dir(dir)? {
        let entry_path = entry?.path();
        if entry_path.is_dir() {
            continue
        }

        let ext = entry_path.extension()
            .and_then(|x| x.to_str())
            .unwrap_or_default();

        println!("ext of {} is {}", entry_path.display(), ext);

        match ext {
            "jpg" | "jpeg" | "png" => images.push(entry_path),
            "mp4" => videos.push(entry_path),
            _ => ()
        };
    }

    // sort images & videos
    images.sort();
    videos.sort();

    
    Ok(MediaEntry {
        dir_path: dir.clone(),
        images: images,
        videos: videos,
    })
}


fn build_index_html(media_entries: Vec<MediaEntry>, base_dir: &str) -> Result<()> {
    let html = vec![
        "<!DOCTYPE html>",
        "<html>",
        "<head>",
        "<meta charset=\"utf-8\">",
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=0\">",
        "<title>Video Index</title>",
        "</head>",
        "<body>",
    ];
    let mut html: Vec<String> = html.into_iter().map(|x| x.to_owned()).collect();
    for entry in media_entries {
        let file_name = entry.dir_path
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or_default();
        let mut entry_html = vec![
            format!("<h1>{}</h1>", file_name),
            "<ul>".to_owned(),
        ];

        for image in entry.images {
            entry_html.push(
                format!("<li><img src=\"{}\" height=\"250em\"></li>", image.strip_prefix(base_dir)?.display())
            );
        }

        for video in entry.videos {
            let file_name = video
                .file_name()
                .and_then(|x| x.to_str())
                .unwrap_or_default();

            entry_html.push(
                format!("<li><a href=\"{}\">{}</li>", video.strip_prefix(base_dir)?.display(), file_name)
            );
        }


        entry_html.extend([
            "</ul>".to_owned()
        ]);
        html.extend(entry_html);
    }

    html.extend([
        "</body>".to_owned(),
        "</html>".to_owned(),
    ]);
    let html = html.join("\n");


    // write to file
    let html_file_path: PathBuf = [base_dir, "index.html"].iter().collect();
    let mut html_file = fs::File::create(html_file_path)?;
    html_file.write_all(html.as_bytes())?;

    Ok(())
}