# crust-archiver

A small command-line archiver written in Rust, built mainly so I could actually understand binary file formats and basic compression instead of just reading about them.

It packs a folder of files into a single custom `.crust` archive and can unpack that archive back into a folder. The format is something I designed myself: a file count header, followed by per-file metadata (name length, name, a compression flag, payload size), followed by the file payloads back to back.

## Why I made this

I wanted to get hands-on with:
- Reading/writing raw bytes in Rust (`u32`/`u16`/`u64` big-endian encoding)
- Implementing Run-Length Encoding (RLE) from scratch
- Per-file compression decisions instead of a blanket one-size-fits-all approach
- Custom error types and `Result`/`?` error propagation
- Basic file I/O without leaning on an existing library for the heavy lifting

It's a learning project first. The format is simple on purpose, and there's a lot it doesn't do yet — see below.

## Usage

```bash
# Pack a directory into an archive
cargo run -- pack <source_dir> <output.crust>

# Unpack an archive into a directory
cargo run -- unpack <archive.crust> <output_dir>
```

Example:

```bash
cargo run -- pack ./my_folder ./backup.crust
cargo run -- unpack ./backup.crust ./restored_folder
```

## How it works (roughly)

Each file gets run through a simple RLE compressor before being written. RLE works well on data with long runs of repeated bytes, but can actually make some files bigger (data with no repeats at all roughly doubles in size under RLE). So before writing each file, the packer compares the compressed size to the original and only keeps the compressed version if it's actually smaller — otherwise it falls back to storing the file raw. A 1-byte flag per file records which path was taken, so unpacking knows whether to run the decompressor or just read the bytes straight through.

Archive layout:

```
[4 bytes]  total file count (u32, big-endian)
for each file:
  [2 bytes]  filename length (u16)
  [N bytes]  filename
  [1 byte]   compression flag (1 = RLE compressed, 0 = stored raw)
  [8 bytes]  payload size (u64)
then, for each file (same order):
  [payload size bytes]  file payload (compressed or raw, depending on the flag)
```

Hidden files (anything starting with `.`, like `.DS_Store`) are skipped automatically while scanning a directory, since they were quietly bloating earlier versions of the archive.

## Known limitations

- Only packs files in the top level of a directory — no recursion into subfolders yet
- RLE is a pretty basic compression scheme, so gains depend entirely on how repetitive the input data is — already-compressed files like PDFs or JPEGs won't shrink much (the raw-fallback logic exists specifically for this case)
- Filenames are stored as full paths at pack time but only the base filename is kept on unpack
- Not tested much against huge files or unusual filenames yet, so treat it as experimental

