# set-luminance

I just wanted to set my external monitor's luminance from the command line.

## Usage

```console
$ set-luminance --help
Set the luminance of my external monitor from the command line, on macOS (Apple Silicon).

Usage: set-luminance <VALUE>

Arguments:
  <VALUE>  Display luminance, from 0 to 100

Options:
  -h, --help     Print help
  -V, --version  Print version
```

i.e.

```console
$ set-luminance 50
```

## How to Contribute

I’ll maintain it as long as it meets my needs, or until I find a better alternative. I’m not looking for contributions, but I’m sharing the code in case it helps someone else. Please feel free to fork it and modify it however you like. I'm not interested in making this:

- more capable
- more configurable
- more user-friendly
- more attractive
- more popular
- GUI-based
- cross-platform (beyond my future use)

There should be similar and/or more capable tools available in every language and platform, so if you have a better option, feel free to keep using that.

## Acknowledgements

[waydabber/m1ddc](https://github.com/waydabber/m1ddc/) and
[haimgel/ddc-macos-rs](https://github.com/haimgel/ddc-macos-rs/) for the inspiration and implementation.

## License

MIT. See [LICENSE](LICENSE).
