# `aoclib`: Utilities convenient for Advent of Code solutions

[Documentation](https://aoclib-docs.netlify.app/aoclib/)

In general, this is intended to provide library support in two broad aspects:
interacting with AoC itself, and making it easier to write solutions.
It is split into several modules to support this.

## Notable AoC Interaction Helpers

Non-exhaustive list; see documentation for more.

- [`aoclib::input::parse`](https://aoclib-docs.netlify.app/aoclib/input/fn.parse.html): parses an input file into an `Iterator<Item=T>` where `T: FromStr`. Doesn't read ahead, for efficiency.

## Notable Solution Support Helpers

The ultimate goal is that any code useful for more than two Advent of Code solutions
of any year gets abstracted into an appropriate module here.

Non-exhaustive list; see documentation for more.

- [`aoclib::geometry`](https://aoclib-docs.netlify.app/aoclib/geometry/index.html): general support modunle for 2d geometry

## Features

The following features exist:

- `map-render`: enables rendering still frames and animations from a map whose tiles implement `ToRgb`. Disabled by default.
