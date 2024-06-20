use axum::{
    extract::Path,
    http::header::{HeaderMap, CONTENT_TYPE},
};
use std::fs;

pub enum StaticFileType {
    Css,
    Js,
    Img,
    Font,
    Other,
}

pub fn extension(filename: &str) -> StaticFileType {
    if filename.contains(".css") {
        StaticFileType::Css
    } else if filename.contains(".js") {
        StaticFileType::Js
    } else if filename.contains(".png") || filename.contains(".jpg") {
        StaticFileType::Img
    } else if filename.contains(".ttf") {
        StaticFileType::Font
    } else {
        StaticFileType::Other
    }
}

#[axum_macros::debug_handler]
pub async fn static_file(Path(path): Path<String>) -> (HeaderMap, Vec<u8>) {
    let file = fs::read("static/".to_owned() + &path);
    let mut header_map = HeaderMap::new();

    match file {
        Ok(file) => {

            match extension(&path) {
                StaticFileType::Css => header_map.insert(CONTENT_TYPE, "text/css".parse().unwrap()),
                StaticFileType::Js => header_map.insert(CONTENT_TYPE, "text/javascript".parse().unwrap()),
                StaticFileType::Img => panic!("images not supported yet."),
                StaticFileType::Font => header_map.insert(CONTENT_TYPE, "font/ttf".parse().unwrap()),
                StaticFileType::Other => panic!("invalid file type requested."),
            };

            (header_map, file)
        },
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    eprintln!("{}\nRequested resource not found: {}", e, &path);
                },
                _ => {
                    eprintln!("Error retrieving resource: {}", e);
                }
            }
            (header_map, Vec::new())
        }
    }

}


#[cfg(test)]
mod tests {
    use crate::staticfiles::{extension, StaticFileType};

    #[test]
    fn css_file_type() {
        assert!(matches!(
            extension("ahhhHHHHHHAHAHHHHH.css"),
            StaticFileType::Css
        ));
        assert!(matches!(
            extension("thisisafilename.js"),
            StaticFileType::Js
        ));
        assert!(matches!(
            extension("veryprettypicture.png"),
            StaticFileType::Img
        ));
        assert!(matches!(
            extension("evenprettierpicture.jpg"),
            StaticFileType::Img
        ));
        assert!(matches!(
            extension("aestheticfonts.ttf"),
            StaticFileType::Font
        ));
    }
}
