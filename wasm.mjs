const wasmPath = "./target/wasm32-unknown-unknown/debug/wasm_debug_check.wasm";

const utf8_decoder = new TextDecoder();

const wasm = await WebAssembly.instantiateStreaming(fetch(wasmPath), {
    env: {
        console_log(ptr, len) {
            console.log(memoryUtf8(ptr, len));
        },
        console_error(ptr, len) {
            console.error(memoryUtf8(ptr, len));
        },
    },
});

/** @typedef {number} usize */

/** @typedef {number} ptr */

/**
 * @typedef Alloc
 * @property {usize} ptr
 * @property {usize} size
 * @property {number} align
 * @property {(start?: usize, end?: usize) => ArrayBuffer} slice
 * @property {() => void} free
 */

const {exports} = wasm.instance;

export {exports};

/**
 * @param {usize} size
 * @param {usize=} align
 * @return {Alloc}
 */
export function alloc(size, align = 1) {
    const ptr = exports.alloc(size, align);
    return {ptr, size, align, slice, free};

    function slice(start = 0, end = size) {
        if (end < 0) {
            end = size + end;
        }
        if (end < 0) {
            end = 0;
        }
        if (end > size) {
            end = size;
        }
        return exports.memory.buffer.slice(ptr + start, ptr + end);
    }

    function free() {
        exports.dealloc(size, align, ptr);
    }
}

/**
 * @param {usize} ptr
 * @param {usize} size
 * @return {ArrayBuffer}
 */
export function memory(ptr, size) {
    exports.memory.buffer.slice(ptr, ptr + size)
}

/**
 * @param {usize} ptr
 * @param {usize} len
 * @return {Uint8Array}
 */
export function memoryU8(ptr, len) {
    return new Uint8Array(exports.memory.buffer, ptr, len);
}

/**
 * @param {usize} ptr
 * @param {usize} len
 * @return {Uint32Array}
 */
export function memoryU32(ptr, len) {
    return new Uint32Array(exports.memory.buffer, ptr, len);
}

export {memoryU32 as memoryUsize};

/**
 * @param {usize} ptr
 * @param {usize} len
 * @return {string}
 */
export function memoryUtf8(ptr, len) {
    return utf8_decoder.decode(memoryU8(ptr, len));
}

/**
 * @param {usize} ptr
 * @param {Uint8Array} bytes
 */
export function write(ptr, bytes) {
    memoryU8(ptr, bytes.length).set(bytes);
}


/**
 * @param {Blob} blob
 * @return {Promise<Alloc>}
 */
export async function allocFromBlob(blob) {
    return await allocFromStream(blob.size, blob.stream());
}

/**
 * @param {usize} size
 * @param {ReadableStream<Uint8Array>} stream
 * @return {Promise<Alloc>}
 */
export async function allocFromStream(size, stream) {
    const result = alloc(size);
    try {
        let ptr = result.ptr;
        for await (const chunk of streamChunks(stream)) {
            if (ptr + chunk.length > result.ptr + size) {
                throw new Error("stream too large: " + ptr + " + " + chunk.length + " > " + result.ptr + " + " + size);
            }
            write(ptr, chunk);
            ptr += chunk.length;
        }
        if (ptr < result.ptr + size) {
            throw new Error("stream too small: " + ptr + " < " + result.ptr + " + " + size);
        }
    } catch (error) {
        result.free();
        throw error;
    }
    return result;
}

// only firefox implements async iterator for ReadableStream already.
/**
 * @template R
 * @param {ReadableStream<R>} stream
 * @returns {AsyncIterableIterator<R>}
 */
async function* streamChunks(stream) {
    const reader = stream.getReader();
    try {
        while (true) {
            const {done, value} = await reader.read();
            if (done) {
                break;
            }
            yield value;
        }
    } finally {
        reader.releaseLock();
    }
}

exports.init();
