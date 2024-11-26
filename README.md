# Velgi

This is a clone of the first level of Velgress from UFO 50,
made for [Eggjam #24][eggjam].
Its main purpose is to test my [Starframe] engine's
physics and lighting features in a "real" context.
You can find screenshots, a Windows build of the game,
and a guide to the controls on [the game's itch.io page][itch].

To build the latest version from source,
install a recent version of Rust and run `cargo run --release`.
You may also need some Vulkan and X11 dependencies on Linux.
For Nix users `flake.nix` should contain all dependencies needed to build on NixOS;
simply run `nix develop` followed by `cargo run --release`.

## License

All source code and assets contained in this repository
are licensed under the [CC0 1.0 Universal Public Domain Dedication][cc0],
meaning you're free to do whatever you want with them.

[eggjam]: https://itch.io/jam/eggjam-24
[itch]: https://molentum.itch.io/velgi
[starframe]: https://github.com/m0lentum/starframe
[cc0]: https://creativecommons.org/publicdomain/zero/1.0/
