```
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   ██████╗ ██╗████████╗   ████████╗██████╗ ███████╗██╗  ██╗    ║
║  ██╔════╝ ██║╚══██╔══╝   ╚══██╔══╝██╔══██╗██╔════╝██║ ██╔╝    ║
║  ██║  ███╗██║   ██║         ██║   ██████╔╝█████╗  █████╔╝     ║
║  ██║   ██║██║   ██║         ██║   ██╔══██╗██╔══╝  ██╔═██╗     ║
║  ╚██████╔╝██║   ██║         ██║   ██║  ██║███████╗██║  ██╗    ║
║   ╚═════╝ ╚═╝   ╚═╝         ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝    ║
║                                                               ║
║              Visual Git Time Travel & File Recovery           ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
```

> *"When did this file get fucked?"* — Every developer, eventually

**git-trek** is a visual file health monitor for git. See your entire codebase as a treemap, scrub through time with your mouse, and instantly spot when files got truncated, deleted, or corrupted. One click to restore.

## The Problem

You're coding. Something breaks. A file got truncated, filled with junk, or mysteriously emptied. Now you need to:
1. Figure out *when* it broke
2. Find what it looked like *before*
3. Restore it

The git CLI way: `git log --oneline -- file`, squint at hashes, `git show abc123:file`, copy-paste... painful.

**git-trek way**: Scroll wheel to scrub time. Red = maybe fucked. Click. Restore. Done.

## Installation

```bash
# Clone and build
git clone https://github.com/junovhs/git-trek.git
cd git-trek
cargo install --path .

# Or just run it
cargo run --release
```

Requires Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Usage

```bash
# In any git repo
git-trek

# Load more history
git-trek --limit 500
```

## The Interface

```
┌─ GIT-TREK v3.0 ─────────────────────────────────────────────────┐
│ [1] Treemap  [2] Heatmap  [3] Minimap  [4] River  [5] Focus     │
├─ 42 / 100 │ fix: restore deleted function ──────────────────────┤
│ ────────────────────────◉───────────────────────────────────────│
├─ Files @ a1b2c3d4 ──────────────────────────────────────────────┤
│┌──────────────────────┐┌────────┐┌────────┐┌──────┐┌───────────┐│
││                      ││        ││   🔴   ││      ││           ││
││    src/app.rs        ││main.rs ││lib.rs  ││cli.rs││ tests/    ││
││      152 ln          ││ 89 ln  ││ 12 ln  ││45 ln ││           ││
││                      ││        ││        ││      ││           ││
│└──────────────────────┘└────────┘└────────┘└──────┘└───────────┘│
├─────────────────────────────────────────────────────────────────┤
│ [click] select | [scroll] time travel | [R] restore | [Q] quit  │
└─────────────────────────────────────────────────────────────────┘
```

**Rectangle size** = file size (lines of code)
**Color** = health status:
- ⬛ Gray: Stable, no significant change
- 🟢 Green: File grew
- 🟡 Yellow: File shrank slightly  
- 🔴 Red: **File shrank >30%** — probably fucked
- 🔵 Blue: New file

## Controls

| Input | Action |
|-------|--------|
| **Scroll wheel** | Scrub through commits |
| **← →** | Navigate timeline |
| **Click file** | Select for restore |
| **Hover** | Highlight file (magenta) |
| **R** | Restore selected file from current commit |
| **1-5** | Switch view modes |
| **Tab** | Cycle views |
| **Esc** | Deselect |
| **Q** | Quit |

## View Modes

| Mode | Purpose | Status |
|------|---------|--------|
| **[1] Treemap** | WinDirStat-style overview | ✅ Working |
| **[2] Heatmap** | Activity over time | 🚧 Coming |
| **[3] Minimap** | Code shape comparison | 🚧 Coming |
| **[4] River** | File size evolution | 🚧 Coming |
| **[5] Focus** | Deep dive on one file | 🚧 Coming |

## How It Works

1. **Loads your git history** — walks commits, records file sizes at each point
2. **Builds a treemap** — files sized proportionally to line count
3. **Tracks health** — compares each commit to its parent, flags suspicious changes
4. **Mouse-driven navigation** — scroll to time travel, click to select, R to restore

No branches created. No working directory changes. Pure read-only inspection until you explicitly restore.

## When To Use It

- **"Something broke, when?"** — Scroll back, watch for red
- **"What did this file look like before?"** — Navigate to commit, click file, R to restore
- **"Overview of my codebase"** — Treemap shows relative file sizes instantly
- **"Which files change together?"** — Scrub time, watch the colors shift

## Options

```bash
git-trek --limit 200    # Load 200 commits (default: 100)
git-trek --help         # Show help
```

## Requirements

- Git repository
- Terminal with mouse support (most modern terminals)
- Rust toolchain (for building)

## Development

```bash
cargo run              # Debug build
cargo run --release    # Fast build  
cargo test             # Run tests
cargo clippy           # Lint
```

## Roadmap

- [x] Treemap view with health coloring
- [x] Mouse hover/click/scroll
- [x] File restore from any commit
- [ ] Heatmap view (activity over time)
- [ ] Minimap view (code shape diff)
- [ ] River view (size evolution)
- [ ] Focus view (single file deep dive)
- [ ] Sparklines per file
- [ ] Filter by path/extension
- [ ] Search commits

---

*Built for developers who think visually and hate typing `git log`.*