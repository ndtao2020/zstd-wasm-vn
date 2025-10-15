#[global_allocator]
pub static GLOBAL_ALLOCATOR: &alloc_cat::AllocCat = &alloc_cat::ALLOCATOR;

use std::io::BufReader;

use wasm_bindgen::prelude::*;
use zstd::stream::{decode_all, encode_all};

const DEFAULT_COMPRESSION_LEVEL: i32 = 6;
const MIN_COMPRESSION_LEVEL: i32 = 1;
const MAX_COMPRESSION_LEVEL: i32 = 22;

/// ZSTD compression and decompression for WebAssembly
#[wasm_bindgen]
pub struct Zstd {}

#[wasm_bindgen]
impl Zstd {
    /// Compresses data using Zstandard compression
    ///
    /// # Arguments
    ///
    /// * `data` - Input data to compress
    /// * `level` - Compression level (1-22, default 3). Higher = better compression but slower
    #[wasm_bindgen]
    pub fn compress(data: &[u8], level: Option<i32>) -> Result<Vec<u8>, JsValue> {
        let compression_level = level.unwrap_or(DEFAULT_COMPRESSION_LEVEL);

        if compression_level < MIN_COMPRESSION_LEVEL || compression_level > MAX_COMPRESSION_LEVEL {
            return Err(JsValue::from_str(&format!(
                "Compression level must be between {} and {}",
                MIN_COMPRESSION_LEVEL, MAX_COMPRESSION_LEVEL
            )));
        }

        encode_all(data, compression_level)
            .map_err(|e| JsValue::from_str(&format!("Compression failed: {}", e)))
    }

    /// Decompresses Zstandard compressed data
    ///
    /// # Arguments
    ///
    /// * `compressed_data` - Zstandard compressed data
    #[wasm_bindgen]
    pub fn decompress(compressed_data: &[u8]) -> Result<Vec<u8>, JsValue> {
        decode_all(compressed_data)
            .map_err(|e| JsValue::from_str(&format!("Decompression failed: {}", e)))
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

/// Streaming compression for large data
#[wasm_bindgen]
pub struct ZstdCompressor {
    encoder: Option<zstd::stream::write::Encoder<'static, Vec<u8>>>,
    level: i32,
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

        let encoder = zstd::stream::Encoder::new(Vec::new(), compression_level)
            .map_err(|e| JsValue::from_str(&format!("Encoder creation failed: {}", e)))?;

        Ok(ZstdCompressor {
            encoder: Some(encoder),
            level: compression_level,
        })
    }

    /// Compresses a chunk of data
    pub fn compress_chunk(&mut self, data: &[u8]) -> Result<(), JsValue> {
        if let Some(encoder) = &mut self.encoder {
            use std::io::Write;
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

    /// Gets the current compression level
    #[wasm_bindgen(getter)]
    pub fn level(&self) -> i32 {
        self.level
    }
}

/// Streaming decompression for large data
#[wasm_bindgen]
pub struct ZstdDecompressor {
    decoder: Option<zstd::stream::read::Decoder<'static, BufReader<std::io::Cursor<Vec<u8>>>>>,
}

#[wasm_bindgen]
impl ZstdDecompressor {
    /// Creates a new streaming decompressor
    #[wasm_bindgen(constructor)]
    pub fn new(compressed_data: &[u8]) -> Result<ZstdDecompressor, JsValue> {
        let cursor = std::io::Cursor::new(compressed_data.to_vec());
        let decoder = zstd::stream::read::Decoder::new(cursor)
            .map_err(|e| JsValue::from_str(&format!("Decoder creation failed: {}", e)))?;

        Ok(ZstdDecompressor {
            decoder: Some(decoder),
        })
    }

    /// Decompresses a chunk of data
    pub fn decompress_chunk(&mut self, max_output_size: usize) -> Result<Vec<u8>, JsValue> {
        if let Some(decoder) = &mut self.decoder {
            let mut buffer = vec![0u8; max_output_size];
            use std::io::Read;
            let bytes_read = decoder
                .read(&mut buffer)
                .map_err(|e| JsValue::from_str(&format!("Decompression failed: {}", e)))?;

            buffer.truncate(bytes_read);
            Ok(buffer)
        } else {
            Err(JsValue::from_str("Decompressor has been finalized"))
        }
    }

    /// Decompresses all remaining data
    pub fn finalize(&mut self) -> Result<Vec<u8>, JsValue> {
        if let Some(decoder) = self.decoder.take() {
            let mut result = Vec::new();
            use std::io::Read;
            decoder
                .finish()
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

    #[test]
    fn test_compress_decompress() {
        let data = b"Hello, World! This is a test string for ZSTD compression.";

        let compressed = Zstd::compress(data, None).unwrap();
        let decompressed = Zstd::decompress(&compressed).unwrap();

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
    fn test_streaming_compression() {
        let data = b"Streaming compression test data".to_vec();
        let chunks = data.chunks(10).collect::<Vec<_>>();

        let mut compressor = ZstdCompressor::new(Some(3)).unwrap();

        for chunk in chunks {
            compressor.compress_chunk(chunk).unwrap();
        }

        let compressed = compressor.finalize().unwrap();
        let decompressed = Zstd::decompress(&compressed).unwrap();

        assert_eq!(data, decompressed);
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
}
