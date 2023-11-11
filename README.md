# `aoclib`: Utilities convenient for Advent of Code solutions

[Documentation](https://aoclib-docs.netlify.app/aoclib/)

In general, this is intended to provide library support in two broad aspects:
interacting with AoC itself, and making it easier to write solutions.
It is split into several modules to support this.

## Notable AoC Interaction Helpers

Non-exhaustive list; see documentation for more.

- [`website::get_input`](https://aoclib-docs.netlify.app/aoclib/website/fn.get_input): get and cache the day's input
- [`aoclib::input::parse`](https://aoclib-docs.netlify.app/aoclib/input/fn.parse.html): parses an input file into an `Iterator<Item=T>` where `T: FromStr`. Doesn't read ahead, for efficiency.

### Automation Disclaimer

This library does follow the [automation guidelines on the /r/adventofcode community wiki](https://www.reddit.com/r/adventofcode/wiki/faqs/automation). Specifically:

- Outbound calls are throttled to every 15 minutes in [`throttle`](https://github.com/coriolinus/aoclib/blob/6af83a4465498eb8344efb178b759298e7b522fa/src/website.rs#L15)
- Once inputs are downloaded, they are cached locally in [`get_input`](https://github.com/coriolinus/aoclib/blob/6af83a4465498eb8344efb178b759298e7b522fa/src/website.rs#L53)
  - If you suspect your input is corrupted, you can request a fresh copy by deleting the existing input and rerunning (subject to throttle)
- The [User-Agent header in `get_input`](https://github.com/coriolinus/aoclib/blob/6af83a4465498eb8344efb178b759298e7b522fa/src/website.rs#L62) is set to this repo; file an issue if abuse is suspected.

## Notable Solution Support Helpers

The ultimate goal is that any code useful for more than two Advent of Code solutions
of any year gets abstracted into an appropriate module here.

Non-exhaustive list; see documentation for more.

- [`aoclib::geometry`](https://aoclib-docs.netlify.app/aoclib/geometry/index.html): general support module for 2d geometry

## Features

The following features exist:

- `map-render`: enables rendering still frames and animations from a map whose tiles implement `ToRgb`. Disabled by default.
