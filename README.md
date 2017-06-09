# run-info
Monitor CPU and memory usage via terminal

![Screenshot](/run-info.png)

## Build
Run `cargo build --release` in the main directory.  
This program will only work on a Linux distribution!

You can also install via `cargo install run-info`.  

## Flags and Options
* `-l` / `--log`				Switch to one-line mode for logging
* `-c` / `--no-color`			Switch to monochrome mode
* `-g` / `--no-graph`			Hide the CPU usage graph
* `-d` / `--delay <ms>`			Set the delay for updating the info and UI (in ms)
* `-s` / `--small` 				Switch to small mode

## Development

I still maintain this project when needed (although I don't know of any bugs, yet).  
Some things I have in mind for possible future updates are:  
* A process view, that only lists the top x processes in terms of CPU and memory usage.
* Moving the printing functions into a general purpose crate
* ~~Ping tests~~. This program lives in https://github.com/mpdrescher/pingtool

If you encounter any bugs or have some feature ideas, please feel free to open an issue.  

## How it works

`main.rs` parses the arguments and maintains the program loop.  
`cpuinfo.rs` basically gets the time the cpu has been busy and the time the cpu has been idling since startup (from `/proc/stat`).  
To get the current cpu load the difference between two timeframes has to be calculated.  
`meminfo.rs` just parses `/proc/meminfo`.  
  
The data is then presented by `printer.rs`, which has a function for each display mode, and `printutils.rs`,
which holds functions that draw the screen elements (the graph, headers, colorized text ...).  
  
Finally, `graph.rs` is a fixed size queue, that is used to buffer the last values.  
  
## Dependencies
`term v.0.4.4`   
`clap v.2.11.0`  
`time v.0.1.35 ` 
