use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, KeyInit};
use aes::Aes128Dec;
use base64::decode;
use byteorder::{ByteOrder, NativeEndian};
use id3::TagLike;
use json::JsonValue;
use phf::{phf_map, Map};
use std::error::Error;
use std::fs::{copy, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Instant;
use std::{mem, process::exit};
use tempfile::NamedTempFile;

type Byte = u8;

pub struct NcmFile {
    output_path: PathBuf,
    meta: JsonValue,
    cover: Vec<u8>,
}

const AES_CORE_KEY: [u8; 16] = [
    0x68, 0x7A, 0x48, 0x52, 0x41, 0x6D, 0x73, 0x6F, 0x35, 0x6B, 0x49, 0x6E, 0x62, 0x61, 0x78, 0x57,
];

const AES_MODIFY_KEY: [u8; 16] = [
    0x23, 0x31, 0x34, 0x6C, 0x6A, 0x6B, 0x5F, 0x21, 0x5C, 0x5D, 0x26, 0x30, 0x55, 0x3C, 0x27, 0x28,
];

static FILTER: Map<&'static str, &'static str> = phf_map! {
        "\\" => "＼",
        "/" => "／",
        ":" => "：",
        "*" => "＊",
        "\"" => "＂",
        "<" => "＜",
        ">" => "＞",
        "|" => "｜",
};

fn byte_read(file: &mut File, length: u32) -> Vec<u8> {
    let mut buff = Vec::with_capacity(length as usize);
    buff.resize(length as usize, 0);
    if let Err(_) = file.read_exact(&mut buff) {
        return vec![];
    };
    buff
}

fn get_data(file: &mut File) -> Vec<u8> {
    let mut buff = [0u8; mem::size_of::<u32>()];
    if let Err(_) = file.read(&mut buff) {
        return vec![];
    };
    byte_read(file, NativeEndian::read_u32(&buff))
}

fn format(vec: Vec<Byte>) -> Vec<[Byte; 16]> {
    let mut buff: [u8; 16] = [0; 16];
    let mut container = Vec::new();

    for (count, value) in vec.iter().enumerate() {
        if (count + 1) % 16 == 0 {
            buff[count % 16] = *value;
            container.push(buff);
            buff = [0; 16]
        } else {
            buff[count % 16] = *value;
        }
    }

    container
}

fn decrypt_aes128(vector: Vec<Byte>, option_key: [Byte; 16]) -> Vec<Byte> {
    let vector_blocks = format(vector);
    let key = GenericArray::from(option_key);
    let cipher = Aes128Dec::new(&key);

    let decrypt_blocks: Vec<_> = vector_blocks
        .iter()
        .map(|block| {
            let mut block_generic = GenericArray::from(*block);
            cipher.decrypt_block(&mut block_generic);
            let buff: Vec<_> = block_generic.to_vec().iter().map(|x| *x).collect();
            buff
        })
        .collect();

    let vec = decrypt_blocks.into_iter().flatten().collect::<Vec<Byte>>();
    let padding = vec[vec.len() - 1] as usize;
    vec[0..(vec.len() - padding)].to_vec()
}

fn skip_length(vector: Vec<Byte>, length: usize) -> Vec<Byte> {
    vector
        .iter()
        .enumerate()
        .filter(|&(count, _)| count >= length)
        .map(|args| *args.1)
        .collect()
}

fn build_key_box(key: Vec<Byte>) -> [u8; 256] {
    let mut key_box = [0; 256];
    for i in 0..256 {
        key_box[i] = i as u8;
    }
    let mut last_byte = 0;
    let mut offset = 0;

    for count in 0..256 {
        let c = ((key_box[count] as u16 + last_byte as u16 + key[offset] as u16) & 0xff) as u8;
        offset += 1;
        if offset >= key.len() {
            offset = 0
        }
        (key_box[c as usize], key_box[count]) = (key_box[count], key_box[c as usize]);
        last_byte = c;
    }

    key_box
}

fn write_in(
    target: &mut PathBuf,
    file: NamedTempFile,
    _file_name: &str,
    _format: &str,
) -> Result<(), Box<dyn Error>> {
    match target.file_name() {
        None => {
            target.push(_file_name);
            target.set_extension(_format);
            let final_target = target;
            copy(file.into_temp_path(), std::path::Path::new(final_target)).expect("Error!");
            Ok(())
        }
        Some(_) => {
            if target.is_dir() {
                target.push(_file_name);
                target.set_extension(_format);
                copy(file.into_temp_path(), std::path::Path::new(target)).expect("Error!");
            } else if target.is_file() {
                target.set_extension(_format);
                copy(file.into_temp_path(), std::path::Path::new(target)).expect("Error!");
            }
            Ok(())
        }
    }
}

impl NcmFile {
    pub fn parse(input: PathBuf, mut output: PathBuf) -> Self {
        let magic_head: [u8; 8] = [0x43, 0x54, 0x45, 0x4e, 0x46, 0x44, 0x41, 0x4d];
        let mut src_file = File::open(&input).expect("Can't open the file!");

        // create the buf to parse data
        let mut buf = [0u8; mem::size_of::<u64>()];

        // judge magic head
        if let Err(e) = src_file.read(&mut buf) {
            println!("Error:{e},can't read head to confirm!");
            exit(-1)
        };

        if buf != magic_head {
            println!("This is not a ncm file!");
            exit(-1)
        }

        if let Err(e) = src_file.seek(SeekFrom::Current(2)) {
            // set offset to move 2 byte
            println!("Error:{e}");
            exit(-1)
        };

        // to parse music file name
        let s = input.file_name().unwrap().to_str().unwrap();
        let mut music_filename = s.get(0..s.len() - 4).unwrap().to_owned();

        for (k, v) in FILTER.into_iter() {
            music_filename = music_filename.replace(*k, *v);
        }

        //163 key parse
        let key_box = build_key_box(skip_length(
            decrypt_aes128(
                get_data(&mut src_file)
                    .into_iter()
                    .map(|value| value ^ 0x64)
                    .collect(),
                AES_CORE_KEY,
            ),
            17,
        ));

        //Music meta info
        let meta: Vec<_> = get_data(&mut src_file)
            .into_iter()
            .map(|value| value ^ 0x63)
            .collect();
        let buff = decode(&meta[22..]).expect("TODO: panic message");
        let meta_info = decrypt_aes128(buff, AES_MODIFY_KEY);
        let info = json::parse(
            std::str::from_utf8(&meta_info[6..]).expect("music info is not valid utf-8:"),
        )
        .expect("error parsing json:");


        let format = info["format"].as_str().unwrap();
        //Music cover data
        if let Err(e) = src_file.seek(SeekFrom::Current(9)) {
            println!("Error:{e}");
            exit(-1)
        };

        let cover = get_data(&mut src_file);

        // Music data
        let mut n: usize = 0x8000;
        let key = key_box.as_slice();
        let mut buffer = [0u8; 0x8000];
        let mut tmp = NamedTempFile::new().expect("error 185");
        let now = Instant::now();
        while n > 1 {
            n = src_file.read(&mut buffer).expect("error 187");
            for i in 0..n {
                let j = (i + 1) & 0xff;
                buffer[i] ^=
                    key[(key[j] as usize + key[(key[j] as usize + j) & 0xff] as usize) & 0xff];
            }
            tmp.write(&buffer).expect("error 193");
        }

        let end = now.elapsed();
        println!("Parse music time:{} micros", end.as_micros());

        write_in(&mut output, tmp, &music_filename, format).expect("Error Happen!");
        NcmFile {
            output_path: output,
            meta: info,
            cover,
        }
    }

    pub fn output(&mut self) -> Result<(), Box<dyn Error>> {
        if self.meta.len() != 0 {
            // judge if metadate is exist
            let music_filename = self.output_path.as_os_str();
            let mut mimetype = "";

            if self.cover.len() != 0 {
                // judge cover data
                let png: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
                if png == &self.cover[..8] {
                    mimetype = "image/png";
                } else {
                    mimetype = "image/jpeg";
                }
            }

            let music_name = self.meta["musicName"].as_str().unwrap();
            let album = self.meta["album"].as_str().unwrap();
            let artist = &self.meta["artist"];
            let _bitrate = self.meta["bitrate"].as_u64().unwrap();
            let _duration = self.meta["duration"].as_u64().unwrap();

            // match music type
            if self.meta["format"].as_str().unwrap() == "mp3" {
                let mut tag = id3::Tag::read_from_path(std::path::Path::new(music_filename))
                    .unwrap_or(id3::Tag::new());
                tag.set_title(music_name);
                tag.set_album(album);
                let mut artists = String::from(artist[0][0].as_str().unwrap());
                for i in 1..artist.len() {
                    artists += "/";
                    artists += artist[i][0].as_str().unwrap();
                }
                tag.set_artist(artists);
                if self.cover.len() != 0 {
                    let picture = id3::frame::Picture {
                        mime_type: mimetype.to_owned(),
                        picture_type: id3::frame::PictureType::CoverFront,
                        description: String::new(),
                        data: self.cover.clone(),
                    };
                    tag.add_frame(picture);
                }
                tag.write_to_path(std::path::Path::new(music_filename), id3::Version::Id3v24)
                    .expect("error writing MP3 file:");
            } else  {
                let mut tag = metaflac::Tag::read_from_path(std::path::Path::new(music_filename))
                    .expect("error reading flac file:");
                let c = tag.vorbis_comments_mut();

                c.set_title(vec![music_name]);
                c.set_album(vec![album]);

                let mut artists: Vec<String> = Vec::new();
                for i in 0..artist.len() {
                    artists.push(artist[i][0].as_str().unwrap().to_string());
                }
                c.set_artist(artists);
                if self.cover.len() != 0 {
                    tag.add_picture(
                        mimetype,
                        metaflac::block::PictureType::CoverFront,
                        self.cover.clone(),
                    );
                }
                tag.write_to_path(std::path::Path::new(music_filename))
                    .expect("error writing flac file:");
            }
        }
        Ok(())
    }
}
