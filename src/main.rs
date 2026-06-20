/*

BECAUSE PDF AND JPEG ARE ALREADY IN A COMPRESSED FORM ie. they already have less no. of consecutive bytes. 
We are using an algo called RLE (Run Length Encoding) , which represents data as pairs of count , byte . 
if hypothetically a data has no consecutive equal bytes then we suffer a 2x increase in the memory. 

However , while compressing we check whether we increase in size , if we do then we dont compress . if we dont then we compress.

*/


#[derive(Debug)]
enum AppCommand {
    Pack { source_dir: String, target_archive: String },
    Unpack { source_archive: String, target_dir: String },
}

// store the AppCommands (all possible variations of it in a enum object). Assign source dir and target_dir to each variant in the enum 

#[derive(Debug)]
enum ArchiverError {
    InvalidHeader, // captures string decoding/format corruption
    IoError(String), // captures io errors
}


use std::env;
use std::fs;
use std::io::Write;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn parse_args() -> Result<AppCommand, ArchiverError> {
    //Gather all arguments from the terminal into a vector of Strings
    // cargo run -- Arguments needed for this program 
    let args: Vec<String> = env::args().collect(); 
    // collects the arguments into a vector of string
    // We need at least 4 items: [program_name, command, path1, path2]
    // Example: ["target/debug/crust", "pack", "my_folder", "output.crust"]
    if args.len() < 4 {
        return Err(ArchiverError::IoError(
            "Error: Missing arguments!\nUsage:\n  crust_archiver pack <source_dir> <output.crust>\n  crust_archiver unpack <archive.crust> <output_dir>".to_string()
        ));
    }

    
    match args[1].as_str() {
        "pack" => {
            Ok(AppCommand::Pack {
                // We .clone() because the Enum needs to own these Strings
                source_dir: args[2].clone(), // If we dont clone them then the vector owns them and they die outside this scope.
                target_archive: args[3].clone(),
            })
        }
        "unpack" => {
            Ok(AppCommand::Unpack {
                source_archive: args[2].clone(),
                target_dir: args[3].clone(),
            })
        }
        // If they typed anything else (like "delete" or "help")
        _ => Err(ArchiverError::IoError(
            format!("Unknown command '{}'. Use 'pack' or 'unpack'.", args[1])
        )),
    }
}



fn get_files_in_dir(dir_path: &str) -> Result<Vec<String>, ArchiverError> {
    let mut file_paths = Vec::new(); // make a new vector to store the filepath

    let entries = fs::read_dir(dir_path).map_err(|e| ArchiverError::IoError(e.to_string()))?;
    // this does a lot of things , first it tries to open a comm channel to the os file manager using fs::read_dir if that fails it gives us a Err variant 
    // std::io::Error , but our fn returns an ArchiverError , we need to convert the error to the apt errortype. 
    // the ? operator is used to early exit out of the function . if the fs::read_dir(...) succeeds then unwrap the Ok result variant. If the execution resulted in Err (Now our Custom Error Variant) then exit out of the function 


    for entry in entries {
        // now read the files (not the content in the files though)
        let entry = entry.map_err(|e| ArchiverError::IoError(e.to_string()))?;
        // since a file read (access) can fail we use map_err to convert the error to our custo Error Variant.
        let path = entry.path();
        if let Some(file_name_os) = path.file_name() {
                if let Some(file_name_str) = file_name_os.to_str() {
                    
                    // SKIP condition: If it starts with a dot, skip it 
                    // ---> System files like .DS_STORE also got picked up , these inflate the file storage.
                    if file_name_str.starts_with('.') {
                        continue; // Jump straight to the next iteration of the loop
                    }
                }
            }
        
        if path.is_file() {
            file_paths.push(path.display().to_string()); // convert a rust PathBuf object into a heap owned String 
        }
    }

    Ok(file_paths) // if everything goes well , return the result variant Ok with the file_path vector . 
}

fn compress_rle(input : &[u8])->Vec<u8>{
    let mut compressed = Vec::new(); 
    if input.is_empty(){return compressed; }
    let mut count = 1 as u8 ; 
    let mut current_byte = input[0]; 
    for byte in &input[1..]{
        if *byte == current_byte && count < u8::MAX{
            count +=1; 
        }
        else {
            compressed.push(count);
            compressed.push(current_byte);
            current_byte = *byte;
            count = 1;
        }
    }
    compressed.push(count);
    compressed.push(current_byte);
    compressed

}

fn decompress_rle(input : &[u8])->Vec<u8>{
    let mut i = 0; 
    let mut decompressed = Vec::new();
    while (i+1<input.len()){
        let count = input[i]; 
        let byte = input[i+1]; 
        for _ in 0..count{
            decompressed.push(byte);
        } 

        i+=2;
    }

    decompressed
}


fn pack_archive(target_path:&str , files:Vec<String>)->Result<(), ArchiverError>{
    /*
    func that writes data to the crust file.
    */

    let mut archive_file = File::create(target_path).map_err(|e| ArchiverError::IoError(e.to_string()))?;
    // makes a new file at the target_path , again mapping the error to our custom error Varient. 
    let total_files = files.len() as u32; 
    let count_bytes: [u8; 4] = total_files.to_be_bytes(); 
    // count_bytes is a vector of 4 elements each of type u8 (unsigned 8) 
    archive_file.write_all(&count_bytes).map_err(|e| ArchiverError::IoError(e.to_string()))?;
    // writing to the archive file 
    // We pass it as a reference (&count_bytes) because write_all wants a byte slice (&[u8])
    // Loop through every file path to write its metadata header
    let mut payloads_to_write = Vec::new(); 

    for file_path in &files {
        // we dont want to lose ownership of the files vector hence passing it as a reference
        // Look up the file's size on the hard drive
        let file_contents = fs::read(file_path)
            .map_err(|e| ArchiverError::IoError(format!("Failed to read for {}: {}", file_path, e)))?;
        
        // attempting compression     
        let compressed_data = compress_rle(&file_contents);

        let (final_payload, compression_flag) = if compressed_data.len() < file_contents.len() {
            (compressed_data, 1u8)  
        } else {
            (file_contents, 0u8)   
        };

        // let file_size: u64   // read the compressed file size
        let payload_size = final_payload.len() as u64;

        // Convert the filename string into a slice of raw bytes
        let name_bytes = file_path.as_bytes();

        let name_len = name_bytes.len() as u16;

        
        
        //Write Name Length (2 bytes)
        archive_file.write_all(&name_len.to_be_bytes())
            .map_err(|e| ArchiverError::IoError(e.to_string()))?;

        //Write the actual Filename bytes (Variable length)
        archive_file.write_all(name_bytes)
            .map_err(|e| ArchiverError::IoError(e.to_string()))?;

        //Write the comp. File Size (8 bytes)
        // archive_file.write_all(&compressed_size.to_be_bytes())
        //     .map_err(|e| ArchiverError::IoError(e.to_string()))?;

        //Write the comp flag
        archive_file.write_all(&[compression_flag])
            .map_err(|e| ArchiverError::IoError(e.to_string()))?;

        archive_file.write_all(&payload_size.to_be_bytes())
            .map_err(|e| ArchiverError::IoError(e.to_string()))?;

        payloads_to_write.push(final_payload);
    }
    println!("Successfully started archive! Wrote file count header: {}", total_files);

    // println!("Appending file payloads...");
    // for file_path in &files {
    //     // Read the entire contents of the source file as raw bytes
    //     let file_contents = fs::read(file_path).map_err(|e| ArchiverError::IoError(format!("Failed to read source file {}: {}", file_path, e)))?;

    //     // Write the raw contents straight into our archive stream
    //     // file_contents is a Vec<u8>, so we pass a reference slice &[u8] using &file_contents
    //     archive_file.write_all(&file_contents).map_err(|e| ArchiverError::IoError(e.to_string()))?;
    // }

    println!("Appending compressed file payloads...");
    for payload in payloads_to_write {
        archive_file.write_all(&payload)
            .map_err(|e| ArchiverError::IoError(e.to_string()))?;
    }

    println!("Successfully completed archive payload packaging!");
    Ok(())
}

fn unpack_archive(archive_path: &str, target_dir: &str) -> Result<(), ArchiverError> {
    //Open the .crust file for reading
    let mut archive_file = File::open(archive_path)
        .map_err(|e| ArchiverError::IoError(e.to_string()))?;

    // Make sure the output destination folder exists on the disk
    // recursively makes all parent dirs if they dont exist , for eg /..../...../
    fs::create_dir_all(target_dir)
        .map_err(|e| ArchiverError::IoError(e.to_string()))?;

    // Read the first 4 bytes to find out how many files are inside
    let mut count_buffer = [0u8; 4];
    archive_file.read_exact(&mut count_buffer)
        .map_err(|e| ArchiverError::IoError("Failed to read archive file count header".to_string()))?;
    // read_exact reads the first n bytes to fill the count_buffer
    // Convert those 4 bytes back into a u32 number
    let total_files = u32::from_be_bytes(count_buffer);
    println!("Unpacker triggered! Archive contains {} files to extract.", total_files);

    // Loop 'total_files' times to parse metadata and extract payloads
    let mut extracted_metadata = Vec::new();
    let mut compression_flag = 0;
    for i in 0..total_files {
        //Read Name Length (2 bytes)
        let mut name_len_buf = [0u8; 2];
        archive_file.read_exact(&mut name_len_buf)
            .map_err(|e| ArchiverError::IoError(format!("Corrupted header reading name length for file {}: {}", i, e)))?;
        let name_len = u16::from_be_bytes(name_len_buf);

        //Read Filename (Variable length based on name_len)
        let mut name_buf = vec![0u8; name_len as usize];
        archive_file.read_exact(&mut name_buf)
            .map_err(|e| ArchiverError::IoError(format!("Corrupted header reading filename for file {}: {}", i, e)))?;
        
        // Convert the byte vector into an actual Rust String safely
        let file_name = String::from_utf8(name_buf)
            .map_err(|e| ArchiverError::InvalidHeader)?; // If it's not valid UTF-8, the file is broken

        let mut flag_buf = [0u8; 1];
        archive_file.read_exact(&mut flag_buf)
            .map_err(|e| ArchiverError::IoError(format!("Corrupted header reading compression flag for file {}: {}", i, e)))?;
        compression_flag = flag_buf[0];


        //Read File Size (8 bytes)
        let mut size_buf = [0u8; 8];
        archive_file.read_exact(&mut size_buf)
            .map_err(|e| ArchiverError::IoError(format!("Corrupted header reading size for file {}: {}", i, e)))?;
        let file_size = u64::from_be_bytes(size_buf);

        println!("Parsed Metadata -> File: '{}', Size: {} bytes", file_name, file_size);

        // Store this metadata map so we can use it to extract the content of the files
        extracted_metadata.push((file_name, file_size));
    }

    println!("Extracting file payloads...");
    for (full_path, file_size) in extracted_metadata {
        // Create a buffer perfectly sized for this file's raw payload content
        let mut payload_buf = vec![0u8; file_size as usize];
        
        // Read exactly 'file_size' bytes out of the archive stream
        archive_file.read_exact(&mut payload_buf)
            .map_err(|e| ArchiverError::IoError(format!("Failed to read payload data: {}", e)))?;


        let final_payload = if compression_flag == 1 {
            println!("Inflating RLE stream for {}...", full_path);
            decompress_rle(&payload_buf)
        } else {
            println!("Reading direct raw stream for {}...", full_path);
            payload_buf // It wasn't compressed, use it exactly as it is!
        };


        // Extract just the plain file name (e.g., "a.txt") from the stored path
        let path_handler = Path::new(&full_path);
        let plain_name = path_handler.file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("extracted_file.dat"); // Fallback name if parsing fails

        // Combine our target directory path with our clean file name
        // Example: ~/Desktop/extracted_files + a.txt -> ~/Desktop/extracted_files/a.txt
        let output_file_path = Path::new(target_dir).join(plain_name);

        // Create the brand new file on your hard drive
        let mut new_file = File::create(&output_file_path)
            .map_err(|e| ArchiverError::IoError(format!("Failed to create output file {:?}: {}", output_file_path, e)))?;

        // Dump the payload bytes into the file
        new_file.write_all(&final_payload)
            .map_err(|e| ArchiverError::IoError(e.to_string()))?;

        println!("Extracted and restored: {:?}", output_file_path);
    }

    println!("All files successfully unpacked and verified!");
    Ok(())
}

fn main() {
    
    let command = match parse_args() {
        Ok(cmd) => {
            println!("Success! Detected command: {:?}", cmd);
            cmd
        }
        Err(err) => {
            eprintln!("Application Error: {:?}", err);
            return;
        }
    };

    match command{
        AppCommand::Pack{source_dir , target_archive}=>{
            println!("Starting pack process from '{}' into '{}'...", source_dir, target_archive);
            match get_files_in_dir(&source_dir){
                Ok(files)=>{
                    println!("Found {} files" , files.len());
                    for file in &files{
                        println!("--->{}" , file);
                    }

                    match pack_archive(&target_archive, files) {
                        Ok(()) => println!("Archiving step 1 complete!"),
                        Err(err) => eprintln!("Archiving failed: {:?}", err),
                    }
                }

                Err(err) => eprintln!("Failed to scan directory: {:?}", err)
            }
        }

        AppCommand::Unpack{source_archive , target_dir}=>{
            println!("Starting unpack process from '{}' into '{}'...", source_archive, target_dir);
            match unpack_archive(&source_archive, &target_dir) {
                Ok(()) => println!("Unpacking complete!"),
                Err(err) => eprintln!("Unpacking failed: {:?}", err),
            }
        }
    }

    
}