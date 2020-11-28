use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use quick_xml::{events::Event, Reader};

#[derive(Debug)]
pub struct TiledLayer {
    pub name: String,
    pub visible: bool,
    pub tiles: Vec<u32>,
}

#[derive(Debug)]
pub struct TiledMapTileset {
    pub first_gid: u32,
    pub source: String,
}

#[derive(Debug)]
pub struct TiledMap {
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub layers: Vec<TiledLayer>,
    pub tilesets: Vec<TiledMapTileset>,
}
#[derive(Debug)]
pub struct TiledTileset {
    tile_width: f32,
    tile_height: f32,
    tile_count: u32,
    images: Vec<TiledTilesetImage>,
}

#[derive(Debug)]
pub struct TiledTilesetImage {
    source: String,
    width: u32,
    height: u32,
}

impl TiledMap {
    pub fn from_bytes(bytes: &[u8]) -> TiledMap {
        let mut reader = Reader::from_reader(bytes);
        reader.trim_text(true);

        let mut tilesets = Vec::new();
        let mut layers = Vec::new();

        let mut buf = Vec::new();

        let mut map_width = None;
        let mut map_height = None;

        let mut map_tilewidth = None;
        let mut map_tileheight = None;

        // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"map" => {
                        for attr in e.attributes() {
                            let a = attr.unwrap();
                            match a.key {
                                b"height" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    map_height = Some(str.parse::<u32>().unwrap());
                                }
                                b"width" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    map_width = Some(str.parse::<u32>().unwrap());
                                }
                                b"tileheight" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    map_tileheight = Some(str.parse::<u32>().unwrap());
                                }
                                b"tilewidth" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    map_tilewidth = Some(str.parse::<u32>().unwrap());
                                }
                                _ => {}
                            }
                        }
                    }
                    b"tileset" => {
                        let first_gid = None;
                        let source = None;
                        for attr in e.attributes() {
                            let a = attr.unwrap();
                            match a.key {
                                b"firstgid" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    let value = str.parse::<u32>().unwrap();
                                    first_gid = Some(value);
                                }
                                b"firstgid" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    source = Some(str);
                                }
                                _ => {}
                            }
                        }
                        tilesets.push(TiledMapTileset {
                            first_gid: first_gid.unwrap(),
                            source: source.unwrap().to_owned(),
                        });
                    }
                    b"layer" => {
                        let mut name = None;
                        let mut visible = true;
                        for attr in e.attributes() {
                            let a = attr.unwrap();
                            match a.key {
                                b"name" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    name = Some(str.to_string());
                                }
                                // b"width" => {
                                //     let str = std::str::from_utf8(&a.value).unwrap();
                                //     width = Some(str.parse::<u16>().unwrap());
                                // }
                                // b"height" => {
                                //     let str = std::str::from_utf8(&a.value).unwrap();
                                //     height = Some(str.parse::<u16>().unwrap());
                                // }
                                b"visible" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    visible = str.parse::<bool>().unwrap();
                                }
                                _ => {}
                            }
                        }

                        let name = name.expect("name not set on layer");
                        let width = map_width.unwrap();
                        let height = map_height.unwrap();

                        let mut encoding = None;

                        let mut state_is_in_data = false;
                        loop {
                            match reader.read_event(&mut buf) {
                                Ok(Event::Start(ref e)) => match e.name() {
                                    b"data" => {
                                        state_is_in_data = true;
                                        for attr in e.attributes() {
                                            let a = attr.expect("data to have attributes");
                                            match a.key {
                                                b"encoding" => {
                                                    let str =
                                                        std::str::from_utf8(&a.value).unwrap();
                                                    encoding = Some(str.to_string());
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                },
                                Ok(Event::End(ref e)) => {
                                    if e.name() == b"data" {
                                        state_is_in_data = false;
                                    } else if e.name() == b"layer" {
                                        break;
                                    }
                                }
                                Ok(Event::Text(ref text)) => {
                                    if state_is_in_data {
                                        match encoding {
                                            Some(ref str) => match str.as_str() {
                                                "base64" => {
                                                    let bytes = base64::decode(
                                                        text.unescape_and_decode_without_bom(
                                                            &reader,
                                                        )
                                                        .unwrap()
                                                        .as_bytes(),
                                                    )
                                                    .expect("malformed layer data");
                                                    let mut rdr = Cursor::new(bytes);
                                                    let mut data = Vec::with_capacity(
                                                        (width * height) as usize,
                                                    );
                                                    for _ in 0..(width * height) {
                                                        data.push(
                                                            rdr.read_u32::<LittleEndian>().unwrap(),
                                                        )
                                                    }

                                                    layers.push(TiledLayer {
                                                        name: name,
                                                        visible: visible,
                                                        tiles: data,
                                                    });
                                                }
                                                _ => {
                                                    panic!("layer data must be encoded with base64")
                                                }
                                            },

                                            None => panic!("layer data must have a set encoding"),
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => (),
                },
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }

        TiledMap {
            width: map_width.expect("map width must be set"),
            height: map_height.expect("map height must be set"),
            tile_width: map_tilewidth.expect("map tile width must me set"),
            tile_height: map_tileheight.expect("map tile height must me set"),
            layers,
            tilesets,
        }
    }
}

impl TiledTileset {
    pub fn from_bytes(bytes: &[u8]) -> TiledTileset {
        let mut reader = Reader::from_reader(bytes);
        reader.trim_text(true);

        let mut images = Vec::new();

        let mut width = None;
        let mut height = None;
        let mut timecount = None;

        let mut buf = Vec::new();

        // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"tileset" => {
                        for attr in e.attributes() {
                            let a = attr.unwrap();
                            match a.key {
                                b"height" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    map_height = Some(str.parse::<u32>().unwrap());
                                }
                                b"width" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    map_width = Some(str.parse::<u32>().unwrap());
                                }
                                b"tileheight" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    map_tileheight = Some(str.parse::<u32>().unwrap());
                                }
                                b"tilewidth" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    map_tilewidth = Some(str.parse::<u32>().unwrap());
                                }
                                _ => {}
                            }
                        }
                    }
                    b"tileset" => {
                        let first_gid = None;
                        let source = None;
                        for attr in e.attributes() {
                            let a = attr.unwrap();
                            match a.key {
                                b"firstgid" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    let value = str.parse::<u32>().unwrap();
                                    first_gid = Some(value);
                                }
                                b"firstgid" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    source = Some(str);
                                }
                                _ => {}
                            }
                        }
                        tilesets.push(TiledMapTileset {
                            first_gid: first_gid.unwrap(),
                            source: source.unwrap().to_owned(),
                        });
                    }
                    b"layer" => {
                        let mut name = None;
                        let mut visible = true;
                        for attr in e.attributes() {
                            let a = attr.unwrap();
                            match a.key {
                                b"name" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    name = Some(str.to_string());
                                }
                                // b"width" => {
                                //     let str = std::str::from_utf8(&a.value).unwrap();
                                //     width = Some(str.parse::<u16>().unwrap());
                                // }
                                // b"height" => {
                                //     let str = std::str::from_utf8(&a.value).unwrap();
                                //     height = Some(str.parse::<u16>().unwrap());
                                // }
                                b"visible" => {
                                    let str = std::str::from_utf8(&a.value).unwrap();
                                    visible = str.parse::<bool>().unwrap();
                                }
                                _ => {}
                            }
                        }

                        let name = name.expect("name not set on layer");
                        let width = map_width.unwrap();
                        let height = map_height.unwrap();

                        let mut encoding = None;

                        let mut state_is_in_data = false;
                        loop {
                            match reader.read_event(&mut buf) {
                                Ok(Event::Start(ref e)) => match e.name() {
                                    b"data" => {
                                        state_is_in_data = true;
                                        for attr in e.attributes() {
                                            let a = attr.expect("data to have attributes");
                                            match a.key {
                                                b"encoding" => {
                                                    let str =
                                                        std::str::from_utf8(&a.value).unwrap();
                                                    encoding = Some(str.to_string());
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                },
                                Ok(Event::End(ref e)) => {
                                    if e.name() == b"data" {
                                        state_is_in_data = false;
                                    } else if e.name() == b"layer" {
                                        break;
                                    }
                                }
                                Ok(Event::Text(ref text)) => {
                                    if state_is_in_data {
                                        match encoding {
                                            Some(ref str) => match str.as_str() {
                                                "base64" => {
                                                    let bytes = base64::decode(
                                                        text.unescape_and_decode_without_bom(
                                                            &reader,
                                                        )
                                                        .unwrap()
                                                        .as_bytes(),
                                                    )
                                                    .expect("malformed layer data");
                                                    let mut rdr = Cursor::new(bytes);
                                                    let mut data = Vec::with_capacity(
                                                        (width * height) as usize,
                                                    );
                                                    for _ in 0..(width * height) {
                                                        data.push(
                                                            rdr.read_u32::<LittleEndian>().unwrap(),
                                                        )
                                                    }

                                                    layers.push(TiledLayer {
                                                        name: name,
                                                        visible: visible,
                                                        tiles: data,
                                                    });
                                                }
                                                _ => {
                                                    panic!("layer data must be encoded with base64")
                                                }
                                            },

                                            None => panic!("layer data must have a set encoding"),
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => (),
                },
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }

        TiledTileset {
            tile_width: tile_width.expect("tile width must me set"),
            tile_height: tile_height.expect("tile height must me set"),
            tile_count: tile_count.expect("tile count must be set"),
            images,
        }
    }
}
