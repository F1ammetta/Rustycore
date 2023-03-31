use audiotags::Tag;
use skytable::actions::Actions;
use skytable::sync::Connection;
use skytable::SkyResult;
use std::fs::read_dir;

pub fn get_song(id: String) -> SkyResult<String> {
    let mut con = Connection::new("127.0.0.1", 2003)?;
    let song: String = con.get(id)?;
    Ok(song)
}

pub fn get_cover(id: String) -> SkyResult<Vec<u8>> {
    let mut con = Connection::new("127.0.0.1", 2003)?;
    let name: String = con.get(id)?;
    let path = r"D:\Users\Sergio\Music\Actual Music\".to_string() + &name;
    let tag = match Tag::default().read_from_path(path) {
        Ok(tag) => tag,
        Err(_) => return Ok(vec![]),
    };
    let cover = match tag.album_cover() {
        Some(cover) => cover,
        None => return Ok(vec![]),
    };
    Ok(cover.data.to_vec())
}

pub fn save(names: &Vec<String>) -> SkyResult<()> {
    let path = r"D:\Users\Sergio\Music\Actual Music\";
    let mut json = String::from("[");
    for name in names.iter() {
        let tag = match Tag::default().read_from_path(path.to_string() + name) {
            Ok(tag) => tag,
            Err(_) => continue,
        };
        let title = tag.title().unwrap_or(name);
        // let title = &title
        //     .chars()
        //     .map(|x| if x == '\0' { ' ' } else { x })
        //     .collect::<String>();
        let artist = tag.artist().unwrap_or("Unknown Artist");
        // let artist = &artist
        //     .chars()
        //     .map(|x| if x == '\0' { ' ' } else { x })
        //     .collect::<String>();
        let album = tag.album_title().unwrap_or("Unknown Album");
        // let album = &album
        //     .chars()
        //     .map(|x| if x == '\0' { ' ' } else { x })
        //     .collect::<String>();
        let dur = tag.duration().unwrap_or(0.) as u32;
        let id = names.iter().position(|x| x == name).unwrap() + 1;
        json += &format!(
            "{{\"title\":\"{}\",\"artist\":\"{}\",\"album\":\"{}\",\"duration\":{},\"id\":{}}},",
            if title.contains(r#"""#) || title.contains(r#"\'"#) {
                title.replace(r#"""#, r#"\""#).replace(r#"\'"#, r#"'"#)
            } else {
                title.to_string()
            },
            if artist.contains(r#"""#) || artist.contains(r#"\'"#) || artist.contains(r#"\0"#) {
                artist.replace(r#"""#, r#"\""#).replace(r#"\'"#, r#"'"#)
            } else {
                artist.to_string()
            },
            if album.contains(r#"""#) || album.contains(r#"\'"#) {
                album.replace(r#"""#, r#"\""#).replace(r#"\'"#, r#"'"#)
            } else {
                album.to_string()
            },
            dur,
            id
        );
    }
    json.pop().unwrap();
    json += "]";
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
                if name == "Find Tomorrow(Ocarina)(Feat.mp3" {
                    continue;
                }
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
