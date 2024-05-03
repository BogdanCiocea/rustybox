use std::env;
use std::io;
use std::process::exit;
use std::fs;
use std::path::Path;
use std::fs::hard_link;
use std::os::unix::fs::symlink;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::Read;

fn touch(path: &str, change_access: bool, no_create: bool, change_modify: bool) -> io::Result<()> {
    let path_obj = Path::new(path);

    if path_obj.exists() {
        if change_access {
            // Simply reading the file changes the access time
            let mut file = File::open(&path_obj)?;
            let mut buffer = Vec::new();
            let _ = file.read_to_end(&mut buffer);
        }

        if change_modify {
            let original_size = path_obj.metadata()?.len();

            // Write a byte to the file to change its modification time
            let mut file = OpenOptions::new().append(true).open(&path_obj)?;
            file.write_all(b"a")?;

            // Truncate the file back to its original size, removing the byte that's added
            file.set_len(original_size)?;
        } else {
			let original_size = path_obj.metadata()?.len();

            // Write a byte to the file to change its modification time
            let mut file = OpenOptions::new().append(true).open(&path_obj)?;
            file.write_all(b"a")?;

            // Truncate the file back to its original size, removing the byte that's added
            file.set_len(original_size)?;
		}
    } else if !no_create {
        // Create the file if it doesn't exist and no_create is false
        OpenOptions::new().write(true).create_new(true).open(path_obj)?;
    }

    Ok(())
}

fn chmod(path: &str, mode: &str) -> i32 {
	// Choose the appropriate function based on the mode
	if is_octal_mode(mode) {
		set_octal_permissions(path, mode)
	} else {
		set_symbolic_permissions(path, mode)
	}
}

fn is_octal_mode(mode: &str) -> bool {
	mode.chars().all(|c| c.is_digit(8))
}

fn set_octal_permissions(path: &str, mode: &str) -> i32 {
	// Convert the mode string to a u32
	let mode_num = match u32::from_str_radix(mode, 8) {
		Ok(m) => m,
		Err(_) => {
			eprintln!("Invalid mode");
			return -25;
		}
	};
	
	// Get the metadata for the file
	if let Err(_) = fs::set_permissions(path, fs::Permissions::from_mode(mode_num)) {
		eprintln!("Failed to set permissions");
		return -25;
	}
	
	0
}

fn set_symbolic_permissions(path: &str, mode: &str) -> i32 {
	// Get the metadata for the file
	let metadata = match fs::metadata(path) {
		Ok(md) => md,
		Err(_) => {
			eprintln!("Invalid command");
			return -25;
		}
	};
	let mut permissions = metadata.permissions();
	let mut current_mode = permissions.mode();

	// Iterate over each character in the mode string
	let chars: Vec<char> = mode.chars().collect();
	let mut idx = 0;
	while idx < chars.len() {
		let mut targets = vec![];
		while let Some(c) = chars.get(idx) {
			if "ugoa".contains(*c) {
				targets.push(*c);
				idx += 1;
			} else {
				break;
			}
		}

		// If no targets were specified, default to 'a'
		if let Some(action) = chars.get(idx) {
			idx += 1;
			while idx < chars.len() && "rwx".contains(chars[idx]) {
				let mask = match chars[idx] {
					'r' => 0o400,
					'w' => 0o200,
					'x' => 0o100,
					_ => 0,
				};
				
				// Update the mode for each target
				for &target in &targets {
					match target {
						'u' => update_mode(*action, &mut current_mode, mask),
						'g' => update_mode(*action, &mut current_mode, mask >> 3),
						'o' => update_mode(*action, &mut current_mode, mask >> 6),
						'a' => {
							update_mode(*action, &mut current_mode, mask);
							update_mode(*action, &mut current_mode, mask >> 3);
							update_mode(*action, &mut current_mode, mask >> 6);
						},
						_ => {}
					}
				}
				idx += 1;
			}
		}
	}

	// Set the permissions for the file
	permissions.set_mode(current_mode);
	if let Err(_) = fs::set_permissions(path, permissions) {
		eprintln!("Failed to set permissions");
		return -25;
	}

	0
}

fn update_mode(action: char, current_mode: &mut u32, mask: u32) {
	// Update the mode based on the action
	match action {
		'+' => *current_mode |= mask,
		'-' => *current_mode &= !mask,
		_ => {}
	}
}

fn rmdir(dirs: &[&str]) -> Result<(), i32> {
	for dir in dirs {
		let path = Path::new(dir);
		if !path.exists() {
			eprintln!("196");
			return Err(-60);
		}
		if path.is_dir() {
			// Remove the directory and all its contents recursively
			fs::remove_dir(path).map_err(|_| -60)?;
		} else {
			// Remove the file
			fs::remove_file(path).map_err(|_| -60)?;
		}
	}

	Ok(())
}

fn cp(source: &str, destination: &str, recursive: bool) -> Result<(), i32> {
	let source_path = Path::new(source);
	let mut destination_path = Path::new(destination).to_path_buf();

	// Check if the source exists
	if !source_path.exists() {
		eprintln!("Invalid command");
		return Err(-90);
	}

	// If the destination is a directory, append the source's filename to it
	if destination_path.is_dir() {
		if let Some(filename) = source_path.file_name() {
			destination_path.push(filename);
		} else {
			eprintln!("Invalid command");
			return Err(-90);
		}

	// If the destination doesn't exist, create the parent directory if needed
	} else if !destination_path.exists() {
		if let Some(parent) = destination_path.parent() {
			if !parent.exists() {
				let _ = fs::create_dir(destination_path.parent().unwrap());
			}
		} else {
			eprintln!("Invalid command");
			return Err(-90);
		}
	}

	// If the source is a file or if recursion is enabled, copy the item
	if source_path.is_file() || recursive {
		if let Err(_e) = copy_item(&source_path, &destination_path, recursive) {
			eprintln!("166");
			Err(-90)
		} else {
			Ok(())
		}
	} else {
		eprintln!("166");
		Err(-90)
	}
}

fn copy_item(source: &Path, destination: &Path, recursive: bool) -> io::Result<()> {
	// If the source is a directory
	if source.is_dir() {
		// Create the directory at the destination if it doesn't exist
		if !destination.exists() {
			fs::create_dir(destination)?;
		}
		// If recursive flag is enabled, process each item in the directory
		if recursive {
			for entry in fs::read_dir(source)? {
				let entry_path = entry?.path();
				let destination_child = destination.join(
					entry_path.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid filename"))?
				);
				copy_item(&entry_path, &destination_child, true)?;
			}
		}
	} 
	// If the source is a file, just copy it to the destination
	else {
		fs::copy(source, destination)?;
	}
	Ok(())
}

fn grep(filenames: &String, pattern: &str) -> Result<(), i32> {
	// Iterate over each filename provided in the filenames string
	for filename in filenames.to_string().split_whitespace() {
		// Attempt to open the file
		let file = match File::open(filename) {
			Ok(f) => f,
			Err(_) => {
				eprintln!("Invalid command");
				return Err(-1);
			}
		};

		// Create a buffered reader for the opened file
		let reader = BufReader::new(file);

		// Iterate over each line in the file, with line numbers
		for (_, line_result) in reader.lines().enumerate() {
			// If the line is successfully read
			if let Ok(line) = line_result {
				// Check if the line contains the pattern or if it ends with the pattern
				// (minus any trailing '$' characters)
				if line.contains(pattern) || line.ends_with(pattern.trim_end_matches('$')) {
					// Print the matching line
					println!("{}", line);
				}
			}
		}
	}

	Ok(())
}

fn rm(files: Vec<String>) -> Result<(), i32> {
	// Flags to determine the behavior of the function
	let mut recursive = false;	// Whether to recursively delete directories
	let mut destroy_dir = false;  // Whether to delete empty directories

	// Check for the presence of flags in the files list
	if files.contains(&"-r".to_string()) || files.contains(&"-R".to_string()) || files.contains(&"--recursive".to_string()) {
		recursive = true;
	}

	if files.contains(&"-d".to_string()) || files.contains(&"--dir".to_string()) {
		destroy_dir = true;
	}

	// Flag to track any errors during file/directory deletion
	let mut error = false;

	// Iterate over each item in the files list
	for file in &files {
		// Skip the item if it's an option (starts with '-')
		if file.starts_with('-') {
			continue;
		}

		// Determine if the item is a directory
		let is_directory = match fs::metadata(&file) {
			Ok(metadata) => metadata.is_dir(),
			Err(_) => {
				eprintln!("186");
				return Err(-70);
			}
		};

		// Handle directory deletion based on the flags
		if is_directory {
			if recursive {
				// Recursively delete the directory
				fs::remove_dir_all(file).map_err(|_| -70)?;
			} else if destroy_dir && fs::read_dir(&file).map_or(false, |mut d| d.next().is_none()) {
				// Delete the directory only if it's empty
				fs::remove_dir(file).map_err(|_| -70)?;
			} else {
				// Flag an error if neither condition is met
				error = true;
			}
		}
		// Handle file deletion
		else {
			fs::remove_file(file).map_err(|_| -70)?;
		}
	}

	// Print an error and return if the error flag is set
	if error {
		eprintln!("186");
		return Err(-70);
	}

	Ok(())
}

fn ls(directory: &str, options: &[&str]) -> Result<(), i32> {
	// Convert the directory string to a Path
	let path = Path::new(directory);

	// Check if the path exists
	if !path.exists() {
		eprintln!("Invalid command");
		return Err(-80);
	}

	// If the path is a file and not a directory, just print the name
	if !path.is_dir() {
		println!("{}", directory);
		return Ok(());
	}

	// Check for recursive option and call the ls_recursive function if found
	for option in options {
		if *option == "-R" || *option == "--recursive" {
			ls_recursive(path, options)?; // NOTE: The ls_recursive function isn't defined in the provided code
			return Ok(());
		}
	}

	// Read the directory entries
	let mut entries = fs::read_dir(path).map_err(|_| -80)?;

	// Iterate over each entry in the directory
	while let Some(entry) = entries.next() {
		let entry = entry.map_err(|_| -80)?;
		// Get the filename of the entry
		let file_name = entry.file_name().into_string().map_err(|_| -80)?;

		// Check if we should display all files (including hidden) or just non-hidden files
		if options.contains(&"-a") || options.contains(&"--all") || !file_name.starts_with('.') {
			println!("{}", file_name);
		}
	}

	Ok(())
}

fn ls_recursive(path: &Path, options: &[&str]) -> Result<(), i32> {
	// Read the directory entries
	let mut entries = fs::read_dir(path).map_err(|_| -80)?;

	// Iterate over each entry in the directory
	while let Some(entry) = entries.next() {
		let entry = entry.map_err(|_| -80)?;
		
		// Get the filename of the entry
		let file_name = entry.file_name().into_string().map_err(|_| -80)?;

		// Print the filename of the entry
		if options.contains(&"-a") || options.contains(&"--all") || !file_name.starts_with('.') {
			println!("{}", file_name);
		}

		// If the entry is a directory, make a recursive call to continue listing its contents
		if entry.file_type().map_err(|_| -80)?.is_dir() {
			ls_recursive(&entry.path(), options)?;
		}
	}

	Ok(())
}

fn ln(source: &str, link_name: &str, is_symbolic: bool) -> i32 {
	// Convert the source and link_name strings to Paths
	let source_path = Path::new(source);
	let link_path = Path::new(link_name);

	// Check if the source file exists
	if !source_path.exists() {
		eprintln!("Invalid command");
		return -50;
	}

	// Ensure the link doesn't already exist
	if link_path.exists() {
		eprintln!("206");
		return -50;
	}

	// Determine the type of link to create based on the is_symbolic flag
	let result = if is_symbolic {
		symlink(source_path, link_path)
	} else {
		hard_link(source_path, link_path)
	};

	// If there's an error in the link creation, print an error and return -50
	if let Err(_) = result {
		eprintln!("206");
		return -50;
	}

	0
}

fn mv(source: &str, destination: &str) -> i32 {
	match fs::rename(source, destination) {
		Ok(_) => 0,
		Err(_) => {
			if copy_and_remove(source, destination) {
				0
			} else {
				eprintln!("216");
				-40
			}
		}
	}
}

fn copy_and_remove(source: &str, destination: &str) -> bool {
	if fs::copy(source, destination).is_err() {
		return false;
	}
	fs::remove_file(source).is_ok()
}

fn pwd() -> io::Result<()> {
	let current_dir = env::current_dir()?;
	println!("{}", current_dir.display());
	Ok(())
}

fn cat(file: &str) -> io::Result<()> {
	let contents = std::fs::read_to_string(file)?;
	print!("{}", contents);
	Ok(())
}

fn mkdir(directory: &str) -> i32 {
	match fs::create_dir(directory) {
		Ok(_) => 0,
		Err(_) => {
			eprintln!("226");
			-30
		}
	}
}

fn echo(args: &[&str]) -> i32 {
	let mut newline = true;
	let mut start_index = 0;

	// Check for the -n option
	if let Some(arg) = args.get(0) {
		if *arg == "-n" {
			newline = false;
			start_index = 1;
		}
	}

	// Print each argument
	for (index, arg) in args.iter().enumerate() {
		if index >= start_index {
			print!("{}", arg);
			if index < args.len() - 1 {
				print!(" ");
			}
		}
	}

	// Print a newline if the -n option wasn't specified
	if newline {
		println!();
	}

	0
}

fn main() {
	let args: Vec<String> = env::args().collect();

	// Check if the user provided a command
	if args.len() < 2 {
		eprintln!("Usage: ./rustybox <command> [args]");
		exit(1);
	}

	// Match the command to the appropriate function
	match args[1].as_str() {
		"pwd" => {
			if let Err(err) = pwd() {
				eprintln!("Error: {}", err);
				exit(1);
			}
		}

		"cat" => {
			if args.len() < 3 {
				eprintln!("236");
				exit(-20);
			}

			for file_arg in args.iter().skip(2) {
				match cat(file_arg) {
					Ok(_) => (),
					Err(_) => {
						eprintln!("236");
						exit(-20);
					}
				}
			}
		}

		"mkdir" => {
			if args.len() < 3 {
				eprintln!("226");
				exit(-30);
			}

			for dir_arg in args.iter().skip(2) {
				let result = mkdir(dir_arg);
				if result == -30 {
					exit(-30);
				}
			}
		}

		"echo" => {
			if args.len() < 2 {
				eprintln!("Invalid command");
				exit(-1);
			}

			let args_str: Vec<&str> = args[2..].iter().map(AsRef::as_ref).collect();

			let result = echo(&args_str);
			if result == -10 {
				eprintln!("246");
			}
			exit(result);
		}

		"mv" => {
			if args.len() != 4 {
				eprintln!("216");
				exit(-40);
			}

			let source = &args[2];
			let destination = &args[3];
			let result = mv(source, destination);
			if result == -40 {
				exit(-40);
			}
		}

		"ln" => {
			if args.len() < 4 {
				eprintln!("206");
				exit(-50);
			}

			let mut is_symbolic = false;
			let mut source = &args[2];
			let mut link_name = &args[3];

			if args[2] == "-s" || args[2] == "--symbolic" {
				is_symbolic = true;
				source = &args[3];
				link_name = &args[4];
			}

			if args[2] == "-a" || args[2] == "--archive" || args[3] == "-a" || args[3] == "--archive" {
				println!("Invalid command");
				exit(-1);
			}

			let result = ln(source, link_name, is_symbolic);
			if result == -50 {
				exit(-50);
			}
		}

		"ls" => {
			let mut directory = ".";
			let mut options: Vec<&str> = Vec::new();
			let mut command_args = args.iter().skip(2);
			
			while let Some(arg) = command_args.next() {
				match arg.as_str() {
					"-a" | "--all" => options.push("-a"),
					"-R" | "--recursive" => options.push("-R"),
					_ => {
						directory = arg;
						break;
					}
				}
			}
			
			let result = ls(directory, &options);
			if result == Err(-80) {
				exit(-80);
			}
		}

		"rm" => {
			let mut files: Vec<String> = Vec::new();
		
			for arg in args.iter().skip(2) {
				files.push(arg.to_string());
			}

			let mut verify_files = false;
			for fie in &files {
				if fie.starts_with('-') {
					continue;
				} else {
					verify_files = true;
					break;
				}
			}

			if !verify_files {
				eprintln!("Invalid command");
				exit(-1);
			}

			let result = rm(files);
			match result {
				Ok(_) => (),
				Err(err_code) =>  {
					exit(err_code);
				}
			}
		},

		"rmdir" => {
			if args.len() < 3 {
				eprintln!("196");
				exit(-60);
			}
	
			let dirs_to_remove: Vec<&str> = args.iter().skip(2).map(AsRef::as_ref).collect();
	
			match rmdir(&dirs_to_remove) {
				Ok(()) => (),
				Err(_code) => {
					eprintln!("196");
					exit(-60);
				}
			}
		}

		"grep" => {
			if args.len() < 4 {
				eprintln!("Usage: ./rustybox grep <pattern> <file1> <file2> ...");
				exit(1);
			}
			
			let pattern = &args[2];
			for filename in &args[3..] {
				if let Err(err) = grep(filename, pattern) {
					eprintln!("{}", err);
					exit(1);
				}
			}
		}

		"chmod" => {
			if args.len() < 4 {
				eprintln!("255");
				exit(-1);
			}
		
			let mode = &args[2];
			let file_path = &args[3];
		
			if mode.starts_with("-") && mode != "-a" {
				eprintln!("255");
				exit(-1);
			}
		
			let result = chmod(file_path, mode);
			if result == -25 {
				exit(-1);
			}
		}

		"cp" => {
			if args.len() < 4 {
				eprintln!("166");
				exit(-90);
			}
		
			let mut source = &args[2];
			let mut destination = &args[3];
			let mut recursive = false;
		
			if source == "-r" || source == "-R" || source == "--recursive" {
				recursive = true;
		
				if args.len() < 5 {
					eprintln!("166");
					exit(-90);
				}
		
				source = &args[3];
				destination = &args[4];
			}
		
			match cp(source, destination, recursive) {
				Ok(_) => (),
				Err(err_code) => exit(err_code),
			}
		}
		"touch" => {
			if args.len() < 2 {
				eprintln!("156");
				std::process::exit(-100);
			}

			let change_access = args.contains(&String::from("-a"));
			let no_create = args.contains(&String::from("-c")) || args.contains(&String::from("--no-create"));
			let change_modify = args.contains(&String::from("-m"));
		
			let file_name = &args[args.len() - 1];
		
			match touch(file_name, change_access, no_create, change_modify) {
				Ok(_) => std::process::exit(0),
				Err(_) => {
					eprintln!("156");
					exit(-100);
				},
			}
		}
		_ => {
			eprintln!("Invalid command");
			exit(-1);
		}
	}
}
