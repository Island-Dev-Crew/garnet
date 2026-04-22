# Garnet brand assets

## Primary logo

`garnet-logo.png` (1024×1024, ~230 KB) — the canonical Garnet image:
half-mechanical-rust lattice (left, Rust rigor), half-faceted-ruby gem
(right, Ruby velocity), luminous neural-graph core (compiler-as-agent).

Note: the file extension is `.png` but the bytes are JPEG-encoded
(standard JFIF). Most tools (browsers, Finder, Preview, GIMP, ImageMagick)
handle both transparently. Strict-format tools (WiX `candle.exe`, Windows
BMP readers) need the conversion steps below.

## Derived assets the installers expect

| Target                            | File                                     | Dimensions | Format |
|-----------------------------------|-------------------------------------------|------------|--------|
| Windows MSI — welcome dialog      | `../wix/Dialog.bmp`                      | 493×312    | BMP    |
| Windows MSI — top banner          | `../wix/Banner.bmp`                      | 493×58     | BMP    |
| Windows MSI — Add/Remove Programs | `../wix/Garnet.ico`                      | multi-res (16/32/48/256) | ICO |
| macOS .pkg — welcome background   | `../macos/resources/background.png`      | 1024×1024 (scaled to window) | PNG |
| README hero                       | `garnet-logo.png` (this dir)             | 1024×1024  | PNG/JPEG |
| Docs-site favicon                 | `../../docs/favicon.ico` (when docs land) | 32×32 + 64×64 | ICO |

The macOS `.pkg` resources (`background.png`) and the README hero are
already in place. The Windows MSI variants require the conversion steps
below before `cargo wix --nocapture` can produce a branded `.msi`.

## Conversion commands (ImageMagick 7+)

```sh
# Install ImageMagick:
#   macOS:       brew install imagemagick
#   Ubuntu:      sudo apt install imagemagick
#   Windows:     winget install ImageMagick.ImageMagick

# From garnet-cli/:
# 1. Dialog.bmp (493×312 — cropped to center the faceted ruby half)
magick assets/garnet-logo.png \
  -resize 493x312^ -gravity center -extent 493x312 \
  -type TrueColor BMP3:wix/Dialog.bmp

# 2. Banner.bmp (493×58 — horizontal strip; same crop rule)
magick assets/garnet-logo.png \
  -resize 493x58^ -gravity center -extent 493x58 \
  -type TrueColor BMP3:wix/Banner.bmp

# 3. Garnet.ico (multi-resolution; pick up 16/32/48/256 from source)
magick assets/garnet-logo.png \
  -define icon:auto-resize=256,48,32,16 wix/Garnet.ico

# 4. LICENSE.rtf for the WiX license-accept step
#    (WiX wants RTF; convert plain LICENSE → LICENSE.rtf via unoconv or pandoc)
pandoc ../LICENSE -o wix/License.rtf
```

Afterward, verify the files with:

```sh
file wix/Dialog.bmp   # → "PC bitmap, Windows 3.x format, 493 x 312 x 24"
file wix/Banner.bmp   # → "PC bitmap, Windows 3.x format, 493 x 58 x 24"
file wix/Garnet.ico   # → "MS Windows icon resource - 4 icons"
```

`BMP3:` is important — WiX rejects BMP4/BMP5 variants.

## Brand color reference

| Use                          | Hex     | ANSI truecolor            |
|------------------------------|---------|---------------------------|
| Primary (wordmark, accents)  | #9C2B2E | `\x1b[38;2;156;43;46m`    |
| Secondary accent (glow)      | #E5C07B | `\x1b[38;2;229;192;123m`  |
| Muted gray (taglines)        | #C8C8D0 | `\x1b[38;2;200;200;208m`  |
| OLED background              | #0A0A0F | `\x1b[48;2;10;10;15m`     |

Used by:
- `garnet --version` / `garnet --help` — primary color on wordmark, muted gray on tagline (when TTY; plain ASCII otherwise — `garnet-cli/src/lib.rs::colored_wordmark`)
- macOS `.pkg` welcome.html / conclusion.html — same palette
- DX comparative deck + paper — see `D_Executive_and_Presentation/`
