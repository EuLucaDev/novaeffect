# Nova Effect app icon

**Creature:** a unicorn, as an engraved head-and-shoulders bust looking out of the tile.

**Style:** an ink engraving portrait in the Nova Foods Apps' owl-template framing (the Nova Film owl):
full-bleed colour field, tight head-and-shoulders crop, the body running off the bottom and right edges.

**Tile:** 512 × 512 viewBox, rounded square `rx=112`, clipped; no border or roundel.

**Palette** (exactly three colours):

| Colour | Hex | Used for |
|---|---|---|
| App colour (Nova Effect pink) | `#e0368f` | the full-bleed field |
| Paper | `#efe9dc` | the figure |
| Ink | `#0b0b0c` | line work and contour |

**Provenance:** the project owner drew it in Nova Foods (a 2880 px engraving-style image, kept in
craftrules at `assets/app-icons/novaeffect/source.png`). It was vectorised with craftrules
`assets/logo-options/_tools/vectorize_tile.py`, which keys out the field and traces the ink with potrace.
Licence: see [LICENSE.txt](LICENSE.txt) (MIT OR Apache-2.0, like the repo).

## Files

| File | What |
|---|---|
| `novaeffect.svg` | master vector, traced at 2048 px (canonical) |
| `novaeffect-small.svg` | lighter vector (~350 KB), traced at 1024 px; also the web favicon |
| `novaeffect-1024.png` | 1024 px render |
| `novaeffect-macos-512.png` | runtime Dock icon on macOS, on Apple's 824/1024 grid |
| `novaeffect.icns` | macOS icon (Apple grid) |
| `novaeffect.ico` | Windows icon, 16–256 px; embedded in `novaeffect.exe` by `apps/novaeffect/build.rs` |
| `hicolor/<n>x<n>/apps/br.com.novafoods.novaeffect.png` | Linux icon theme, 16–512 px; the 256 px one is the runtime icon on Windows and Linux |
| `hicolor/scalable/apps/br.com.novafoods.novaeffect.svg` | Linux scalable icon |

App ID: `br.com.novafoods.novaeffect` (Wayland app ID, `packaging/linux/br.com.novafoods.novaeffect.desktop`).

## Regenerate

Copy a new `icon-master.svg` / `icon.svg` from craftrules `assets/app-icons/novaeffect/` over
`novaeffect.svg` / `novaeffect-small.svg`, then run `packaging/icons.sh` (needs `resvg`; `iconutil`
on macOS for the `.icns`). Copy `novaeffect-small.svg` to `apps/novaeffect-web/web/favicon.svg` too.
