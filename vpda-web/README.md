# VPDA Web Shell

This is a fresh web shell built from the layout and styling direction in:

- `/Users/mac/RustroverProjects/vpda/src/template/ttemplate.html`

It intentionally does **not** use the old outer `template/` folder.

## What is already in place

- `Market Watch` workspace
  - full-screen chart area
  - left watchlist
  - drawing tool rail
  - chart toggles for `Grid`, `Crosshair`, `Heatmap`
  - right inspector panel
- `Forecast` workspace
  - full-screen analysis layout
  - asset / target / custom date / bias controls
  - ranked weekly profile bars
  - key levels table
  - VPDA and VM scenario sections
  - warnings + context cards
- `Shell` workspace
  - dedicated command console
  - command entry row
- modal host
  - activation placeholder
  - warning/info/error modal plumbing

## Current implementation mode

The frontend now prefers a real local Rust API first:

- `http://127.0.0.1:3017/api/startup-state`
- `http://127.0.0.1:3017/api/watchlist`
- `http://127.0.0.1:3017/api/market`
- `http://127.0.0.1:3017/api/forecast`
- `http://127.0.0.1:3017/api/shell`

If that API is not running, the bridge falls back to mock frontend data so the shell still opens.

The bridge lives in:

- `/Users/mac/RustroverProjects/vpda/vpda-web/src/bridge/vpdaBridge.ts`

The Rust API entrypoint lives in:

- `/Users/mac/RustroverProjects/vpda/src/web_api.rs`

## Run It

1. Start the Rust API:
   - `cargo run -- --web-api`
2. In another terminal, start the web shell:
   - `cd /Users/mac/RustroverProjects/vpda/vpda-web`
   - `npm install`
   - `npm run dev`
3. Open the local Vite URL shown in the terminal.

## What is wired already

1. `startup-state`
   - startup validation
   - weekend lock warning
   - activation/update/offline warning states

2. `watchlist`
   - real watchlist rows for supported instruments

3. `market`
   - real candle loading from Rust
   - timeframe support: `1H`, `4H`, `1D`, `1W`
   - stable heatmap levels derived in Rust

4. `forecast`
   - runs the real `FORECAST DAILY` backend path
   - parses structured bias / confidence / weekly profile / levels / VPDA scenarios / VM scenarios / warnings / context

5. `shell`
   - real `FORECAST ...` and `CURRENT STATE ...` commands through the Rust bridge

## Recommended next integration order

1. replace the current HTTP bridge with Tauri commands/events
2. stream forecast output progressively instead of waiting for the full result
3. add activation submit flow from the modal
4. replace the placeholder drawing rail with a real overlay layer on top of `lightweight-charts`

## Notes

- The visual system keeps the same font family and the same color language from `ttemplate.html`.
- The noisy scanline overlay and over-styling were intentionally left out.
- The shell is responsive and keeps the chart as the dominant workspace in Market Watch.
