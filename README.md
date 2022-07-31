# ugly

`ugly` is a collection of not-invented-here abstractions and tools for doing graphics things like font rendering.
It was spun out of the [zombiesplit](https://github.com/ashikun/zombiesplit) project, and will generally expand to serve the needs of `zsclient`.

_Note:_ `ugly` is in pre-release (v0.x), and its API _will_ change and break between minor release versions.
If using `ugly`, make sure you pin to a particular minor release.

## Features

### Current

- Proportional ASCII pixel font renderer
- Convenience functionality for metrics (points, sizes, rectangles)
- Rectangular fills
- Targets SDL

### Planned

- Other, perhaps more low-level, graphics backends
- Text user interface (eg, use ncurses etc as a backend for `ugly`)
- Better documentation
- Primitives for slightly more sophisticated graphics drawing
- Possibly abstraction layers for keyboard and mouse input (currently zombiesplit handles the former directly and the latter not at all)
- Performance improvements (I've done a bit of work on the font renderer, but it's still not the most optimal thing)

### Not planned

- Unicode support, at least not for a while (this might change eventually if there is strong demand for it)

## Licence

MIT.

## What does `ugly` stand for?

'Undead graphics library', or something like that.