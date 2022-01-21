# Oxygen Voice Journal

Oxygen is a voice journal and audio analysis toolkit for people who want to
change the way their voice comes across. Or rather, it will be. This is still
in very early development. 

You can watch me code this at https://www.youtube.com/c/JocelynStericker

## Motivation

Like many others, I couldn't stand my voice, but it's not until fairly recently
that I learned just how flexible our vocal system is and took advantage of
that. With the risk of being a little bold, with enough training, it's possible
for most people to develop their voice to take on the character of another
voice they like. The main limitation is that it can be physically impossible to
develop a darker, more boomy voice than your vocal tract supports, but
otherwise, whatever traits you wish your voice had, you can probably build
them!

Voice training feels a bit like a game. The "core loop" is recording a sample,
analyzing it, both with my ear and with software, trying to change some aspect
of it, and then repeating. Progress is slow, so it's also important to be able
to look back weeks or months to see improvement and stay motivated. Software
can help visualize or measure the aspects we need to work on, but my current
setup, without that software, is a bit of a Rube Goldberg machine, and I hope I
can make training more accessible, organized, and addicting.

## Dependencies

Building Oxygen requires [Rust](https://www.rust-lang.org/learn/get-started), [Node 16](https://nodejs.org/en/), and [CMake](https://cmake.org/download/) on all platforms. The Linux build also requires ALSA development files.

### Linux

I recommend installing Rust using [rustup](https://www.rust-lang.org/learn/get-started).

Oxygen is tested with Node 16. If your package manager contains Node 16, I recommend using that. Otherwise, see [Installing Node.js via package manager](https://nodejs.org/en/download/package-manager/).

I recommend installing CMake via your package manager.

In addition, on Linux, the ALSA development files are required. These are provided as part of the `libasound2-dev` package on Debian and Ubuntu distributions and `alsa-lib-devel` on Fedora.

### macOS

I recommend installing Node and CMake using brew (`brew install node cmake`), and Rust using [rustup](https://www.rust-lang.org/learn/get-started).

### Windows

I recommend installing all dependencies from the download links on their homepages.

When installing Rust and Node on Windows, please follow the instructions in the respective installers to install associated build tools.

## Building the CLI

This project uses Rust. Once Rust is installed, you can build and run the
oxygen CLI with cargo:

```
cd ./core
cargo run
```

## Using the CLI

You can run the CLI by running `cargo run` in `./core`.

Oxygen will store clips in the "oxygen.sqlite" file in the current working
directory.

Oxygen supports the following commands:

```
cargo run -- record [name]
  Record an audio clip using the default input device until ctrl+c is pressed.
  If name is not specified, the current date and time will be used.

cargo run -- list
  List all clips

cargo run -- record [name]

cargo run -- play name
  Play the clip with the given name

cargo run -- delete name
  Delete the clip with the given name

cargo run -- import path [name]
  Import the clip at the given path. If a name is not specified, the clip will be
  named after the path.

cargo run -- export name path
  Export the clip with the given name to the given path.
  The path should end in ".wav".

cargo run -- export-all folder
  Export all clips to the given folder.
```

## Running the UI

To run the UI:

```
cd ./ui
npm install
npm start
```

## Building the app in release mode

To build a package to `ui/out`:

```
cd ./ui
npm run prod:package
```

To create the kind of asset that would get uploaded to GitHub releases:

```
cd ./ui
npm run prod:make
```

`npm run prod:publish` would publish the app to GitHub releases, but it requires `GITHUB_TOKEN` to be set. We only publish releases from GitHub Actions.

## Contributing

Bug fixes are very welcome! I do have a rough roadmap planned for this project,
and am uploading development recordings, so reach out (e.g., in issues) before
implementing any major features.
