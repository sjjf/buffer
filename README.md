# buffer
A simple tool to decouple two programs on the command line. buffer simply reads from stdin into an internal buffer, and at the same time writes the buffer to stdout, with the two sides being handled by separate threads.

This was "inspired" by the vulnerability outlined at https://www.idontplaydarts.com/2016/04/detecting-curl-pipe-bash-server-side/ - buffer acts as a protection against this vulnerability (verified against the demo code available from that site). However, you should still never directly pipe code from the Internet into a shell, because that's just plain stupid.

buffer is written in rust. To build and run it you should go to https://www.rust-lang.org, download the appropriate installer, install it, and then build buffer by running 'cargo build' in the top of the source tree (the same directory as the Cargo.toml file). The executable will be under the target/ directory.
