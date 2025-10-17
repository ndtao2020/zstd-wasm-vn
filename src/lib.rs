#[global_allocator]
pub static GLOBAL_ALLOCATOR: &alloc_cat::AllocCat = &alloc_cat::ALLOCATOR;

use std::io::{BufReader, Cursor, Read, Write};
use wasm_bindgen::prelude::*;
use zstd::{
    dict::{DecoderDictionary, EncoderDictionary},
    stream::{read::Decoder, write::Encoder},
};

const DEFAULT_COMPRESSION_LEVEL: i32 = 6;
const MIN_COMPRESSION_LEVEL: i32 = 1;
const MAX_COMPRESSION_LEVEL: i32 = 22;

/// Compresses data using Zstandard compression
///
/// # Arguments
///
/// * `data` - Input data to compress
/// * `level` - Compression level (1-22, default 6). Higher = better compression but slower
#[wasm_bindgen]
pub fn compress(data: &[u8], level: Option<i32>) -> Result<Vec<u8>, JsValue> {
    let compression_level = level.unwrap_or(DEFAULT_COMPRESSION_LEVEL);

    if compression_level < MIN_COMPRESSION_LEVEL || compression_level > MAX_COMPRESSION_LEVEL {
        return Err(JsValue::from_str(&format!(
            "Compression level must be between {} and {}",
            MIN_COMPRESSION_LEVEL, MAX_COMPRESSION_LEVEL
        )));
    }

    zstd::stream::encode_all(data, compression_level)
        .map_err(|e| JsValue::from_str(&format!("Compression failed: {}", e)))
}

/// Compresses data using Zstandard compression with a dictionary
///
/// # Arguments
///
/// * `data` - Input data to compress
/// * `dict` - The compression dictionary
/// * `level` - Compression level (1-22, default 6). Higher = better compression but slower
#[wasm_bindgen]
pub fn compress_with_dict(
    data: &[u8],
    dict: &[u8],
    level: Option<i32>,
) -> Result<Vec<u8>, JsValue> {
    let compression_level = level.unwrap_or(DEFAULT_COMPRESSION_LEVEL);

    if compression_level < MIN_COMPRESSION_LEVEL || compression_level > MAX_COMPRESSION_LEVEL {
        return Err(JsValue::from_str(&format!(
            "Compression level must be between {} and {}",
            MIN_COMPRESSION_LEVEL, MAX_COMPRESSION_LEVEL
        )));
    }

    let mut results = Vec::<u8>::new();
    let dict_trained = EncoderDictionary::copy(dict, compression_level);
    let mut encoder = match Encoder::with_prepared_dictionary(&mut results, &dict_trained) {
        Ok(d) => d,
        Err(e) => {
            return Err(JsValue::from_str(&e.to_string()));
        }
    };

    if let Err(err) = encoder.write_all(data) {
        return Err(JsValue::from_str(&err.to_string()));
    }
    if let Err(err) = encoder.finish() {
        return Err(JsValue::from_str(&err.to_string()));
    }

    Ok(results)
}

/// Decompresses Zstandard compressed data
///
/// # Arguments
///
/// * `compressed_data` - Zstandard compressed data
#[wasm_bindgen]
pub fn decompress(compressed_data: &[u8]) -> Result<Vec<u8>, JsValue> {
    zstd::stream::decode_all(compressed_data)
        .map_err(|e| JsValue::from_str(&format!("Decompression failed: {}", e)))
}

/// Decompresses Zstandard compressed data using a dictionary
///
/// # Arguments
///
/// * `data` - Zstandard compressed data
/// * `dict` - The decompression dictionary (must match the compression dictionary)
#[wasm_bindgen]
pub fn decompress_with_dict(data: &[u8], dict: &[u8]) -> Result<Vec<u8>, JsValue> {
    let dict_trained = DecoderDictionary::copy(dict);

    let mut decoder = match Decoder::with_prepared_dictionary(data, &dict_trained) {
        Ok(d) => d,
        Err(e) => {
            return Err(JsValue::from_str(&e.to_string()));
        }
    };

    let mut results = Vec::<u8>::new();

    if let Err(err) = decoder.read_to_end(&mut results) {
        return Err(JsValue::from_str(&err.to_string()));
    }

    decoder.finish();

    Ok(results)
}

/// ZSTD compression and decompression for WebAssembly
#[wasm_bindgen]
pub struct Zstd {}

#[wasm_bindgen]
impl Zstd {
    #[wasm_bindgen]
    pub fn compress(data: &[u8], level: Option<i32>) -> Result<Vec<u8>, JsValue> {
        compress(data, level)
    }

    #[wasm_bindgen]
    pub fn compress_with_dict(
        data: &[u8],
        dict: &[u8],
        level: Option<i32>,
    ) -> Result<Vec<u8>, JsValue> {
        compress_with_dict(data, dict, level)
    }

    #[wasm_bindgen]
    pub fn decompress(compressed_data: &[u8]) -> Result<Vec<u8>, JsValue> {
        decompress(compressed_data)
    }

    #[wasm_bindgen]
    pub fn decompress_with_dict(compressed_data: &[u8], dict: &[u8]) -> Result<Vec<u8>, JsValue> {
        decompress_with_dict(compressed_data, dict)
    }

    /// Returns the recommended default compression level
    #[wasm_bindgen(js_name = defaultCompressionLevel)]
    pub fn default_compression_level() -> i32 {
        DEFAULT_COMPRESSION_LEVEL
    }

    /// Returns the minimum compression level
    #[wasm_bindgen(js_name = minCompressionLevel)]
    pub fn min_compression_level() -> i32 {
        MIN_COMPRESSION_LEVEL
    }

    /// Returns the maximum compression level
    #[wasm_bindgen(js_name = maxCompressionLevel)]
    pub fn max_compression_level() -> i32 {
        MAX_COMPRESSION_LEVEL
    }

    /// Estimates the compressed size for planning purposes
    #[wasm_bindgen(js_name = compressBound)]
    pub fn compress_bound(input_size: usize) -> usize {
        zstd::zstd_safe::compress_bound(input_size)
    }

    /// Calculates compression ratio (smaller = better compression)
    #[wasm_bindgen(js_name = compressionRatio)]
    pub fn compression_ratio(original_size: usize, compressed_size: usize) -> f64 {
        if original_size == 0 {
            return 0.0;
        }
        compressed_size as f64 / original_size as f64
    }

    /// Calculates space savings percentage
    #[wasm_bindgen(js_name = spaceSavings)]
    pub fn space_savings(original_size: usize, compressed_size: usize) -> f64 {
        if original_size == 0 {
            return 0.0;
        }
        (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0
    }
}

/// ==================================== [Streaming] ====================================
/// Streaming compression for large data
#[wasm_bindgen]
pub struct ZstdCompressor {
    encoder: Option<Encoder<'static, Vec<u8>>>,
}

#[wasm_bindgen]
impl ZstdCompressor {
    /// Creates a new streaming compressor
    #[wasm_bindgen(constructor)]
    pub fn new(level: Option<i32>) -> Result<ZstdCompressor, JsValue> {
        let compression_level = level.unwrap_or(DEFAULT_COMPRESSION_LEVEL);

        if compression_level < MIN_COMPRESSION_LEVEL || compression_level > MAX_COMPRESSION_LEVEL {
            return Err(JsValue::from_str(&format!(
                "Compression level must be between {} and {}",
                MIN_COMPRESSION_LEVEL, MAX_COMPRESSION_LEVEL
            )));
        }

        let encoder = Encoder::new(Vec::new(), compression_level)
            .map_err(|e| JsValue::from_str(&format!("Encoder creation failed: {}", e)))?;

        Ok(ZstdCompressor {
            encoder: Some(encoder),
        })
    }

    /// Compresses a chunk of data
    pub fn compress_chunk(&mut self, data: &[u8]) -> Result<(), JsValue> {
        if let Some(encoder) = &mut self.encoder {
            encoder
                .write_all(data)
                .map_err(|e| JsValue::from_str(&format!("Compression failed: {}", e)))?;

            Ok(())
        } else {
            Err(JsValue::from_str("Compressor has been finalized"))
        }
    }

    /// Finalizes compression and returns the compressed data
    pub fn finalize(&mut self) -> Result<Vec<u8>, JsValue> {
        if let Some(encoder) = self.encoder.take() {
            encoder
                .finish()
                .map_err(|e| JsValue::from_str(&format!("Finalization failed: {}", e)))
        } else {
            Err(JsValue::from_str("Compressor has already been finalized"))
        }
    }
}

/// Streaming decompression for large data
#[wasm_bindgen]
pub struct ZstdDecompressor {
    // The Decoder itself implements `Read` to yield UNCOMPRESSED data.
    // The inner `Cursor` holds the COMPRESSED input bytes.
    // We use a Box<dyn Read> for the inner reader to handle the complexity
    // of the lifetime and the fact that Decoder::new wraps the Cursor in a BufReader.
    // However, the original structure with BufReader<Cursor<Vec<u8>>> is fine for Wasm
    // where we often just pass the whole compressed byte array.
    decoder: Option<Decoder<'static, BufReader<Cursor<Vec<u8>>>>>,
}

#[wasm_bindgen]
impl ZstdDecompressor {
    /// Creates a new streaming decompressor
    #[wasm_bindgen(constructor)]
    pub fn new(compressed_data: &[u8]) -> Result<ZstdDecompressor, JsValue> {
        // Use Cursor to treat the Vec<u8> as a stream (implements io::Read)
        let cursor = Cursor::new(compressed_data.to_vec());

        // Decoder::new automatically wraps the reader in a BufReader for efficiency
        let decoder = Decoder::new(cursor)
            .map_err(|e| JsValue::from_str(&format!("Decoder creation failed: {}", e)))?;

        Ok(ZstdDecompressor {
            decoder: Some(decoder),
        })
    }

    /// Decompresses a chunk of data
    pub fn decompress_chunk(&mut self, max_output_size: usize) -> Result<Vec<u8>, JsValue> {
        // The Option is only None if `stream_to_end` or an equivalent consuming method was called.
        let decoder = self
            .decoder
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Decompressor has been finalized/consumed."))?;

        // 1. Prepare output buffer
        let mut buffer = vec![0u8; max_output_size];

        // 2. Read decompressed data from the decoder
        let bytes_read = decoder
            .read(&mut buffer)
            .map_err(|e| JsValue::from_str(&format!("Decompression failed: {}", e)))?;

        // 3. Truncate buffer to actual read size
        buffer.truncate(bytes_read);

        Ok(buffer)
    }

    /// Decompresses all remaining data and consumes the decoder.
    pub fn finalize(&mut self) -> Result<Vec<u8>, JsValue> {
        if let Some(mut decoder) = self.decoder.take() {
            let mut result = Vec::new();
            decoder
                .read_to_end(&mut result)
                .map_err(|e| JsValue::from_str(&format!("Final read failed: {}", e)))?;

            Ok(result)
        } else {
            Err(JsValue::from_str("Decompressor has already been finalized"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DICT: &[u8] = b"aaaaa";

    #[test]
    fn test_compress_decompress() {
        let data = b"Hello, World! This is a test string for ZSTD compression.";

        let compressed = Zstd::compress(data, None).unwrap();
        let decompressed = Zstd::decompress(&compressed).unwrap();

        assert_eq!(data, decompressed.as_slice());
    }

    #[test]
    fn test_compress_decompress_with_dict() {
        let data = b"Hello, World! This is a test string for ZSTD compression.";

        let compressed = compress_with_dict(data, DICT, None).unwrap();
        let decompressed = decompress_with_dict(&compressed, DICT).unwrap();

        assert_eq!(data, decompressed.as_slice());
    }

    #[test]
    fn test_compress_levels() {
        let data = b"Test data for compression level testing".repeat(100);

        for level in 1..=6 {
            let compressed = Zstd::compress(&data, Some(level)).unwrap();
            let decompressed = Zstd::decompress(&compressed).unwrap();
            assert_eq!(data, decompressed.as_slice());
        }
    }

    #[test]
    fn test_compression_ratio() {
        let original_size = 1000;
        let compressed_size = 250;
        let ratio = Zstd::compression_ratio(original_size, compressed_size);

        assert_eq!(ratio, 0.25);
    }

    #[test]
    fn test_space_savings() {
        let original_size = 1000;
        let compressed_size = 250;
        let savings = Zstd::space_savings(original_size, compressed_size);

        assert_eq!(savings, 75.0);
    }

    #[test]
    fn test_compress_bound() {
        let input_size = 1000;
        let bound = Zstd::compress_bound(input_size);

        assert!(bound >= input_size);
    }

    #[test]
    fn test_streaming_compression() {
        let data = b"Streaming compression test data".to_vec();

        let mut compressor = ZstdCompressor::new(Some(DEFAULT_COMPRESSION_LEVEL)).unwrap();

        let chunks = data.chunks(10).collect::<Vec<_>>();
        for chunk in chunks {
            compressor.compress_chunk(chunk).unwrap();
        }
        let compressed = compressor.finalize().unwrap();

        assert_eq!(data, Zstd::decompress(&compressed).unwrap());
    }

    #[test]
    fn test_chunked_decompression() {
        let data = b"This is some test data that is intentionally longer to ensure streaming works across 
        multiple blocks, which is necessary to properly test the chunking logic of the ZstdDecompressor implementation.".to_vec();

        let compressed = Zstd::compress(&data, Some(DEFAULT_COMPRESSION_LEVEL)).unwrap();

        let mut decompressor =
            ZstdDecompressor::new(&compressed).expect("Failed to create decompressor");

        let chunk_size = 10; // Request a small chunk size to force multiple calls
        let mut decompressed: Vec<u8> = Vec::new();

        loop {
            let chunk = decompressor
                .decompress_chunk(chunk_size)
                .expect("Decompress chunk failed");

            // Stop condition: empty chunk indicates EOF
            if chunk.is_empty() {
                break;
            }

            // Collect the chunk
            decompressed.extend_from_slice(&chunk);
        }

        // Verify
        assert_eq!(data, decompressed);
    }
}
