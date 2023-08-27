extern crate cc;

fn main() {
	// Define the compiler and flags
	let mut build = cc::Build::new();
	build.cpp(false); // Treat input files as C code
	build.flag("-g"); // Compiler flags

	// Define include directories
	build.include("src");

	let src = "obdii/src/";

	// Define source files
	let library_src_files = [format!("{src}OBDII.c"), format!("{src}OBDIICommunication.c")];

	let daemon_src_files = [format!("{src}OBDIIDaemon.c")];

	let cli_src_files = [format!("{src}OBDII.c"), format!("{src}OBDIICommunication.c")];

	// Compile the shared library
	build.files(&library_src_files).shared_flag(true).compile("obdii");

	// Compile the daemon
	build.files(&daemon_src_files).compile("obdiid");

	// Compile the CLI
	build.files(&cli_src_files).compile("cli");
}
