# Unbranded stand-ins

When the story needs "the old way" or a category other products occupy (an editor, a spreadsheet,
a chat window), draw a generic one. Never show another company's product, logo, screenshot, icon
or recognisable layout.

## Draw it

- Simple flat SVG: a grey window outline, a tab row, a few blurred bars for text. Enough to
  read as "an editor", never enough to read as one brand.
- Split it into layers that move separately (a window, the back and the front of a bin), so a
  graphic shot can put one between the others.
- Shapes only: paths, rectangles, circles, ellipses, lines, polylines and polygons, with fill,
  stroke, opacity and group transforms. Convert text to paths; graphic layers draw no text.

## Animate it as a graphic shot

```yaml
shots:
  - id: old-way
    from: start
    graphic:
      layers:
        - svg: graphics/window.svg
          width: 0.62                    # of the frame's shorter side, so it fits every aspect
          keys:
            - { at: 0.0, x: 0.5, y: 0.42 }
            - { at: 0.1, rotate: -6, squash: 0.9, ease: out-cubic }
            - { at: 0.44, y: 0.72, rotate: 160, scale: 0.16, squash: 1.0, arc: 0.3 }
```

Each key sets position (`x`, `y`, 0–1 of the frame), `rotate` (degrees), `scale`, `squash`
(1 keeps the shape; area is kept), `opacity`, an `ease` (`linear`, `in-out`, `out-cubic`,
`in-cubic`) and an `arc` that bows the path upward for a throw. `at` is seconds from the start of
the shot. The schema is `schema/film.schema.json`.

For example, `examples/fired-ide/` throws a drawn editor window into a bin in half a second.
