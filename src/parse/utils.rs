use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, KeyInit};
use aes::Aes128Dec;
use byteorder::{ByteOrder, NativeEndian};
use std::error::Error;
use std::fs::{copy, File};
use std::io::Read;
use std::mem;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

type Byte = u8;

fn bytes_read(file: &mut File, length: u32) -> Vec<u8> {
    let mut buff = Vec::with_capacity(length as usize);
    buff.resize(length as usize, 0);
    if let Err(_) = file.read_exact(&mut buff) {
        return vec![];
    };
    buff
}

pub fn get_data(file: &mut File) -> Vec<u8> {
    let mut buff = [0u8; mem::size_of::<u32>()];

    if let Err(_) = file.read(&mut buff) {
        return vec![];
    };

    bytes_read(file, NativeEndian::read_u32(&buff))
}

pub fn decrypt_aes128(vector: Vec<Byte>, option_key: [Byte; 16]) -> Vec<Byte> {
    let vector_blocks = {
        let mut buff: [u8; 16] = [0; 16];
        let mut container = Vec::new();

        for (count, value) in vector.iter().enumerate() {
            if (count + 1) % 16 == 0 {
                buff[count % 16] = *value;
                container.push(buff);
                buff = [0; 16]
            } else {
                buff[count % 16] = *value;
            }
        }

        container
    };

    let key = GenericArray::from(option_key);
    let cipher = Aes128Dec::new(&key);

    // To decrypt aes block
    let decrypt_blocks: Vec<_> = vector_blocks
        .iter()
        .map(|block| {
            let mut block_generic = GenericArray::from(*block);
            cipher.decrypt_block(&mut block_generic);
            let buff: Vec<_> = block_generic.to_vec().iter().map(|x| *x).collect();
            buff
        })
        .collect();

    // To remove aes padding len
    let vec = decrypt_blocks.into_iter().flatten().collect::<Vec<Byte>>();
    let padding = vec[vec.len() - 1] as usize;
    vec[0..(vec.len() - padding)].to_vec()
}

pub fn build_key_box(key: Vec<Byte>) -> [u8; 256] {
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

pub fn write_in(
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
            copy(file.into_temp_path(), Path::new(final_target)).expect("Error!");
            Ok(())
        }
        Some(_) => {
            if target.is_dir() {
                target.push(_file_name);
                target.set_extension(_format);
                copy(file.into_temp_path(), Path::new(target)).expect("Error!");
            } else if target.is_file() {
                target.set_extension(_format);
                copy(file.into_temp_path(), Path::new(target)).expect("Error!");
            }
            Ok(())
        }
    }
}
