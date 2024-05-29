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

#[cfg(test)]
mod tests {
    use crate::staticfiles::{extension, StaticFileType};

    #[test]
    fn css_file_type() {
        assert!(matches!(
            extension("ahhhHHHHHHAHAHHHHH_awewrjwlejrlwjerj__bruhhhh.css"),
            StaticFileType::Css
        ));
    }
}
