# fundsp-example

A simple example of using the Rust library
[FunDSP](https://github.com/SamiPerttu/fundsp) to synthesis audio. This is not
an interactive synthesiser, it will only play a hard coded sound, although this
library does support real-time applications. If you'd like to run or modify the
code, you'll need to:

* Install the Rust programming language
  (https://www.rust-lang.org/tools/install)
* Download this repository, either by cloning with Git or downloading a zip
  file. click `code` at the top of this page to do that.
* Navigate to the folder you've downloaded and run the following command:
  `cargo run --release`. This might take a bit of time the first time you run
  this command, as cargo will download all the dependencies. After this is
  complete, you should hear a tone (A 440 Hz).
* Use a text editor to change the code in `src/main.rs`.
