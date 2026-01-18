use lz4_flex::{compress_prepend_size, decompress_size_prepended};

pub fn compress(buf: &[u8]) -> Vec<u8> {
    compress_prepend_size(buf)
}

pub fn decompress(buf: &[u8]) -> Vec<u8> {
    decompress_size_prepended(buf).unwrap()
}
