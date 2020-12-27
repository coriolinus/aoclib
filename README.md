# `aoclib`: Utilities convenient for Advent of Code solutions

[Documentation](https://aoclib-docs.netlify.app/aoclib/)

In general, this is intended to provide library support in two broad aspects:
interacting with AoC itself, and making it easier to write solutions.
It is split into several modules to support this.

## AoC Interaction Modules

- `config`: AoC configuration which stores the session cookie and any local
  configuration data, such as custom paths.
- `input`: provides utilities for parsing input files: `parse`, `parse_newline_sep`, and `CommaSep`.
- `website`: provides functions for getting a day's URL and downloading an input file

## Solution Support Modules

The ultimate goal is that any code useful for more than two Advent of Code solutions
of any year gets abstracted into an appropriate module here.

- `geometry`: provides general support for 2d geometry. There are sub-modules:
  - `direction`: provides a 2d cartesian direction
  - `line_segment`: provides a bounded line segment of indefinite location: direction and distance
  - `line`: provides a bounded line segment of definite location: endpoint and endpoint
  - `map`: provides a 2d cartesian grid map, abstract over the tile type
  - `tile`: provides the `DisplayWidth` trait which helps a `Map` work with a custom tile type, and the `Bool` enum which can be used for conventional boolean maps in the AoC style
  - `point`: provides a 2d point `Point` and `PointTrait` abstracting some useful methods on a point
  - `vector3`: provides a 3d point `Vector3` which impls `PointTrait`
  - `vector4`: provides a 4d point `Vector4` which impls `PointTrait`
- `numbers`: provides numeric algorithms
  - `chinese_remainder`: given a set of constraints of the form `n % C == M`, finds a `n`
    which satisfies those constraints

## Features

This module is not currently split into distinct features. Depending on observed
compile times, that may change in the future.
