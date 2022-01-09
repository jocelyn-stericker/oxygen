# Oxygen Voice Journal

Oxygen is a voice journal and audio analysis toolkit for people who want to
change the way their voice comes across. Or rather, it will be. This is still
in very early development. For now, it's a command line version of the
Voice Memos app on Apple platforms.

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

## Building

This project uses Rust. Once Rust is installed, you can build and run oxygen
with cargo:

```
cargo run
```

## Using

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

## Contributing

Bug fixes are very welcome! I do have a rough roadmap planned for this project,
and am uploading development recordings, so reach out (e.g., in issues) before
implementing any major features.
