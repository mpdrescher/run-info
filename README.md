# run-info
Monitor CPU and memory usage via terminal

![Screenshot](/run-info.png)

# Build
Run `cargo build --release` in the main directory.  
This program will only work on a Linux distribution!

You can also install via `cargo install run-info`.  

# Flags and Options
* `-l` / `--log`				-Switch to one-line mode for logging
* `-c` / `--no-color`			-Switch to monochrome mode
* `-g` / `--no-graph`			-Hide the CPU usage graph
* `-d` / `--delay <ms>`			-Set the delay for updating the info and UI (in ms)
* `-s` / `--small` 				-Switch to small mode

# Dependencies
"term" v.0.4.4  
"clap" v.2.11.0  
"time" v.0.1.35  
