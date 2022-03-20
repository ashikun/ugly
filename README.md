# ugly

`ugly` (Undead Graphics LibrarY) is (will be) a Rust library for handling various lowish-level graphical user interface tasks.
It was spun out of the [zombiesplit](https://github.com/ashikun/zombiesplit) project.

## Features

### Current

- Proportional high-ASCII pixel font renderer
- Convenience functionality for metrics (points, sizes, rectangles)
- Rectangular fills
- Targets SDL

### Planned

- Text user interface
- Primitives for slightly more sophisticated graphics drawing
- Possibly abstraction layers for keyboard and mouse input (currently zombiesplit handles the former directly and the latter not at all)

### Not planned

- Unicode support, at least not for a while (this might change eventually if there is strong demand for it)

## Licence

MIT.
