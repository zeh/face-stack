# Face Stack

A simple command line application in Rust to create a stack of face images.

Unless you're debugging something, I recommend running with `--release` so everything is faster.

* Run: `cargo run --release`
* Run with parameters: `cargo run --release -- --input /something/*.jpg --size 1024x1024 --face-scale 0.5 --opacity 0.9 --seed 123 --output result.jpg --crop-width 50% --crop-height 60%`
* See basic parameters: `cargo run --release -- --help`

Some properties accept multiple values, with weights.

* `--opacity`: accepts a value like `0.9`, a range like `0.7-0.8`, and multiple values/ranges like `0.5 0.6 0.8-0.9`, including with weights, like `0.5@10 0.6` (`0.5` is 10 times more likely to be picked than `0.6`)
* `--crop-width` and `--crop-height`: accepts a value like `0` (for pixels), a value like `50%` (for percentage), a range (mixed or not) like `10-50%`, and multiple values/ranges (also with weights) like `20 30 10%-500@2`
