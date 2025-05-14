# Face Stack

A simple command line application in Rust to create a stack of face images.

Unless you're debugging something, I recommend running with `--release` so everything is faster.

* Run: `cargo run --release`
* Run with parameters: `cargo run --release -- --input /something/*.jpg --size 1024x1024 --face-scale 0.5 --output result.jpg`
