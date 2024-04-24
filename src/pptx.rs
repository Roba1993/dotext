use zip::ZipArchive;

use xml::events::Event;
use xml::reader::Reader;

use std::clone::Clone;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use zip::read::ZipFile;

use crate::document::{Document, DocumentKind};

pub struct Pptx {
    data: Cursor<String>,
}

impl Document<Pptx> for Pptx {
    fn kind(&self) -> DocumentKind {
        DocumentKind::Pptx
    }

    fn from_reader<R>(reader: R) -> io::Result<Pptx>
    where
        R: Read + io::Seek,
    {
        let mut archive = ZipArchive::new(reader)?;

        let mut xml_data = String::new();

        for i in 0..archive.len() {
            let mut c_file = archive.by_index(i).unwrap();
            if c_file.name().starts_with("ppt/slides") {
                let mut _buff = String::new();
                c_file.read_to_string(&mut _buff);
                xml_data += _buff.as_str();
            }
        }

        let mut buf = Vec::new();
        let mut txt = Vec::new();

        if xml_data.len() > 0 {
            let mut to_read = false;
            let mut xml_reader = Reader::from_str(xml_data.as_ref());
            loop {
                match xml_reader.read_event(&mut buf) {
                    Ok(Event::Start(ref e)) => match e.name() {
                        b"a:p" => {
                            to_read = true;
                            txt.push("\n".to_string());
                        }
                        b"a:t" => {
                            to_read = true;
                        }
                        _ => (),
                    },
                    Ok(Event::Text(e)) => {
                        if to_read {
                            let text = e.unescape_and_decode(&xml_reader).unwrap();
                            txt.push(text);
                            to_read = false;
                        }
                    }
                    Ok(Event::Eof) => break, // exits the loop when reaching end of file
                    Err(e) => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "Error at position {}: {:?}",
                                xml_reader.buffer_position(),
                                e
                            ),
                        ))
                    }
                    _ => (),
                }
            }
        }

        Ok(Pptx {
            data: Cursor::new(txt.join("")),
        })
    }
}

impl Read for Pptx {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.data.read(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn instantiate() {
        let _ = Pptx::open(Path::new("samples/sample.pptx"));
    }

    #[test]
    fn read() {
        let mut f = Pptx::open(Path::new("samples/sample.pptx")).unwrap();

        let mut data = String::new();
        let len = f.read_to_string(&mut data).unwrap();
        println!("len: {}, data: {}", len, data);
    }
}
