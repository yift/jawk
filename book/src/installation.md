# Getting jawk
There are a few ways to install `jawk`:
## From source
To install `jawk` from source, make sure you have the Rust toolchain installed. See details in [here](https://www.rust-lang.org/tools/install)
### From the repository
To install `jawk` from the repository, one need to clone the repository, build the tool and copy the executable to the path.
For example, on linux (assuming `~/bin` is in the path):
```
git clone https://github.com/yift/jawk
cd jawk
cargo build -r
cp target/release/jawk ~/bin
```

### From Cargo
To install `jawk` from Cargo, one can simply run:
```
cargo install jawk
```

## From Docker
One can use [`jawk` docker container](https://hub.docker.com/r/yiftach/jawk). Please note that this will not allow you to access any local files (unless you add them to the container volumes).
For example:
```
echo '{"a": 1}{"a": 10}{"a": 32}' | docker run -i --rm yiftach/jawk --select '.a=a' -o csv
```
(To install docker see [here](https://docs.docker.com/engine/install/)).

## From binary
Some operating system binaries are available in:
* [Linux GNU](jawk-x86_64-unknown-linux-gnu.zip)
* [Windows](jawk-x86_64-pc-windows-msvc.zip)
* [Linux musl](jawk-x86_64-unknown-linux-musl.zip)
* [Mac OS](jawk-x86_64-apple-darwin.zip)
