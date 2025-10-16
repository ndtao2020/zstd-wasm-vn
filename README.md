# zstd-wasm-vn
[![npm](https://img.shields.io/npm/v/zstd-wasm-vn)](https://www.npmjs.com/package/zstd-wasm-vn)

## Browser

### Bulk (One-shot)

For simple, full-buffer compression/decompression.

```js
import init, { Zstd } from 'zstd-wasm-vn/web';

await init();

const str = "Compresses data using Zstandard compression";

const data = new TextEncoder().encode(str);
const compressed = Zstd.compress(data, 6);
const decompressed = Zstd.decompress(compressed);

console.log("Original String: ", str);
console.log("Decompressed String: ", new TextDecoder().decode(decompressed));
```

### Streaming

For processing large files read from a drag-and-drop event or an HTTP fetch stream.

```js
import init, { ZstdCompressor, ZstdDecompressor } from 'zstd-wasm-vn/web';

await init();

// --- Decompression Stream Example (Browser File Input) ---
// Assuming you have the full compressed Uint8Array (e.g., loaded from a file input)
async function streamDecompress(compressedData) {
    const decompressor = new ZstdDecompressor(compressedData);
    const CHUNK_SIZE = 1024 * 128; // Request 128KB chunks

    const decompressedChunks = [];
    
    while (true) {
        // 1. Decompress a chunk
        const chunk = decompressor.decompress_chunk(CHUNK_SIZE);
        
        // 2. Stop condition
        if (chunk.length === 0) {
            break;
        }

        // 3. Collect the chunk for processing
        decompressedChunks.push(chunk);
        
        // Optionally, yield or process the chunk here
        // console.log(`Processed ${chunk.length} bytes`);
    }

    // Combine all chunks into a final array
    const totalLength = decompressedChunks.reduce((sum, chunk) => sum + chunk.length, 0);
    const finalData = new Uint8Array(totalLength);
    let offset = 0;
    for (const chunk of decompressedChunks) {
        finalData.set(chunk, offset);
        offset += chunk.length;
    }
    
    console.log(`Total decompressed bytes: ${finalData.length}`);
    return finalData;
}

// Example usage with placeholder compressed data
// const compressedFileBytes = await fetch('data.zst').then(r => r.arrayBuffer()).then(b => new Uint8Array(b));
// streamDecompress(compressedFileBytes);
```

## Node.js

### Bulk (One-shot)

For simple, full-buffer compression/decompression.

```js
const { randomBytes } = require('node:crypto');
const { Zstd } = require('zstd-wasm-vn/nodejs');

// Generate 32 bytes of random data
const data = randomBytes(32); 

// Compress the entire buffer at level 6
const compressed = Zstd.compress(data, 6);

// Decompress the entire buffer
const decompressed = Zstd.decompress(compressed);

// Verify
console.log('Match:', data.equals(decompressed));
```

### Streaming

For handling large files or continuous network data without loading the entire content into memory.

```js
const { ZstdCompressor, ZstdDecompressor } = require('zstd-wasm-vn/nodejs');
const { createReadStream, createWriteStream } = require('node:fs');

// --- Compression Stream Example ---

// 1. Create a stream compressor instance (level 3)
const compressor = new ZstdCompressor(3);

createReadStream('large_input.txt', { highWaterMark: 64 * 1024 }) // Read 64KB chunks
    .on('data', (chunk) => {
        // 2. Compress each chunk
        compressor.compress_chunk(chunk);
    })
    .on('end', () => {
        // 3. Finalize and get the full compressed output
        const compressedData = compressor.finalize(); 
        console.log(`Compressed size: ${compressedData.length} bytes`);
        // Write the result to a file (optional)
        // createWriteStream('output.zst').write(compressedData);
    });

// --- Decompression Stream Example ---

// Assuming 'compressed_data' is a Buffer containing the Zstd file content
const compressedData = /* ... load compressed data Buffer here ... */;
const decompressor = new ZstdDecompressor(compressedData);

let totalDecompressedSize = 0;
const CHUNK_SIZE = 1024 * 64; // Request 64KB uncompressed chunks

while (true) {
    // 1. Decompress a chunk
    const chunk = decompressor.decompress_chunk(CHUNK_SIZE);

    // 2. Stop condition (empty chunk means EOF)
    if (chunk.length === 0) {
        break;
    }

    // 3. Process/write the uncompressed chunk
    // createWriteStream('decompressed.txt').write(chunk);
    totalDecompressedSize += chunk.length;
}

console.log(`Total decompressed size: ${totalDecompressedSize} bytes`);
```
