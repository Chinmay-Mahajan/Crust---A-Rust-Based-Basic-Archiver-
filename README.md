# crust-archiver

A tiny command-line archiver written in Rust, mostly as a way for me to actually understand binary file formats instead of just reading about them.

It packs a folder of files into a single custom `.crust` archive, and can unpack that archive back into a folder. No compression, no encryption — just a straightforward binary layout I designed myself: a file count header, followed by per-file metadata (name length, name, size), followed by the raw file contents back to back.

## Why I made this

I wanted to get comfortable with:
- Reading/writing raw bytes in Rust (`u32`/`u16`/`u64` big-endian encoding)
- Custom error types and `Result`/`?` error propagation
- Basic file I/O without leaning on an existing archive crate

It's not meant to compete with `tar` or `zip` — it doesn't compress anything, doesn't handle nested directories yet, and the format is about as simple as it gets. Think of it as a learning exercise that happens to work.

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

## How the format works (roughly)

```
[4 bytes]  total file count (u32, big-endian)
for each file:
  [2 bytes]  filename length (u16)
  [N bytes]  filename
  [8 bytes]  file size (u64)
then, for each file (same order):
  [file size bytes]  raw file contents
```

## Known limitations

- Only packs files in the top level of a directory — no recursion into subfolders yet
- No compression, so archives will generally be larger than the originals combined
- Filenames are stored as full paths at pack time but only the base filename is kept on unpack
- Not really tested against huge files or weird filenames, so don't trust it with anything important yet

## Possible next steps

- Recursive directory packing
- Maybe a checksum per file to catch corruption
- Compression (probably the most useful missing piece)


