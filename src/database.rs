#![allow(dead_code)]
use audiotags::Tag;
use dotenv::dotenv;
use id3::Tag as Id3Tag;
use metaflac::Tag as FlacTag;
use mp4ameta::Tag as M4aTag;
use serde::{Deserialize, Serialize};
use skytable::actions::Actions;
use skytable::sync::Connection;
use skytable::SkyResult;
use std::fs::read_dir;

#[derive(Serialize, Deserialize, Debug)]
struct Song {
    title: String,
    artist: String,
    album: String,
    duration: u32,
    lyrics: String,
    id: usize,
}

pub fn get_song(id: String) -> SkyResult<String> {
    let mut con = Connection::new("127.0.0.1", 2003)?;
    let song: String = con.get(id)?;
    Ok(song)
}

pub fn get_cover(id: String) -> SkyResult<Vec<u8>> {
    dotenv().ok();
    let dir = std::env::var("MUSIC_DIR").unwrap();
    let mut con = Connection::new("127.0.0.1", 2003)?;
    let name: String = con.get(id)?;
    let path = dir + "\\" + &name;
    let tag = match Tag::default().read_from_path(path) {
        Ok(tag) => tag,
        Err(_) => return Ok(vec![]),
    };
    let cover = match tag.album_cover() {
        Some(cover) => cover,
        None => return Ok(std::fs::read("def-cover.png").unwrap()),
    };
    Ok(cover.data.to_vec())
}

pub fn save(names: &Vec<String>) -> SkyResult<()> {
    dotenv().ok();
    let path = std::env::var("MUSIC_DIR").unwrap() + "\\";
    let mut songs: Vec<Song> = Vec::new();
    for name in names.iter() {
        let tag = match Tag::default().read_from_path(path.to_string() + name) {
            Ok(tag) => tag,
            Err(_) => continue,
        };
        let mut lyrics: String = "".to_string();
        let def_lyrics = vec![String::from("")];
        if name.ends_with(".flac") {
            let flac = FlacTag::read_from_path(path.to_string() + name).unwrap();
            let lyricss = match flac.vorbis_comments().unwrap().get("LYRICS") {
                Some(lyricss) => lyricss,
                None => &def_lyrics,
            };
            lyrics = lyricss[0].to_string();
        }

        if name.ends_with(".m4a") {
            let m4a = M4aTag::read_from_path(path.to_string() + name).unwrap();
            lyrics = match m4a.lyrics() {
                Some(lyrics) => lyrics.to_string(),
                None => "".to_string(),
            };
        }

        if name.ends_with(".mp3") {
            let mp3 = Id3Tag::read_from_path(path.to_string() + name).unwrap();
            let ls = mp3.lyrics();
            ls.for_each(|l| lyrics.push_str(&l.text));
        }
        lyrics = lyrics
            .replace("\"", "\\\"")
            .replace("\n", "\\n")
            .replace("\'", "'");

        if lyrics.contains("\n") {
            println!("asdf");
        }

        let title = tag.title().unwrap_or(name);
        let artist = tag.artist().unwrap_or("Unknown Artist");
        let album = tag.album_title().unwrap_or("Unknown Album");
        let dur = tag.duration().unwrap_or(0.) as u32;
        let id = names.iter().position(|x| x == name).unwrap() + 1;
        let song = Song {
            title: title.to_string(),
            artist: artist.to_string(),
            album: album.to_string(),
            duration: dur,
            lyrics,
            id,
        };
        songs.push(song);
    }
    let json: String = match serde_json::to_string(&songs) {
        Ok(json) => json,
        Err(_) => "".to_string(),
    };
    std::fs::write("music.json", json)?;
    println!("JSON Dumped!");

    Ok(())
}

pub fn update_db() -> SkyResult<()> {
    let names = get_names().unwrap();
    clear().unwrap();
    populate(&names).unwrap();

    let songs = match fetch() {
        Ok(songs) => songs,
        Err(_) => vec![],
    };
    save(&songs).unwrap();
    Ok(())
}

pub fn get_names() -> SkyResult<Vec<String>> {
    let path = r"D:\Users\Sergio\Music\Actual Music\";
    let iter = read_dir(path)?;
    let mut names: Vec<String> = Vec::new();
    for entry in iter {
        let entry = entry?;
        let path = entry.path();
        match path.file_name().unwrap().to_str() {
            Some(name) => {
                if name.ends_with(".mp3") || name.ends_with(".flac") || name.ends_with(".m4a") {
                    names.push(name.to_string());
                    name.to_string()
                } else {
                    continue;
                }
            }
            None => continue,
        };
    }
    Ok(names)
}

pub fn fetch() -> SkyResult<Vec<String>> {
    let mut con = Connection::new("127.0.0.1", 2003)?;
    let mut names: Vec<String> = Vec::new();
    let length: String = match con.get("length") {
        Ok(v) => v,
        Err(_) => {
            return Ok(names);
        }
    };
    let n: usize = match length.parse() {
        Ok(v) => v,
        Err(_) => {
            return Ok(names);
        }
    };
    for i in 1..=n {
        let name: String = match con.get(i.to_string()) {
            Ok(v) => v,
            Err(_) => {
                continue;
            }
        };
        names.push(name);
    }
    Ok(names)
}

pub fn clear() -> SkyResult<()> {
    let mut con = Connection::new("127.0.0.1", 2003)?;
    con.flushdb()?;
    Ok(())
}

pub fn populate(names: &Vec<String>) -> SkyResult<()> {
    let mut con = Connection::new("127.0.0.1", 2003)?;
    let names_in_db: Vec<String> = fetch()?;
    let mut inserted: usize = 1;
    for i in 1..=names.len() {
        con.del("length")?;
        con.set("length", names.len().to_string())?;
        if names_in_db.contains(&names[i - 1]) {
            continue;
        }
        con.set(
            (names_in_db.len() + inserted).to_string(),
            names[i - 1].clone().to_string(),
        )?;
        println!(
            "Inserted: {} at index {}",
            names[i - 1],
            names_in_db.len() + inserted
        );
        inserted += 1;
    }
    Ok(())
}
