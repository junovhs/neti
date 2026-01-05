slopchop on  main is 📦 v1.3.4 via 🦀 v1.91.1
❯ cargo install --path . --force
  Installing slopchop v1.3.4 (C:\Users\junov\slopchop)
    Updating crates.io index
     Locking 108 packages to latest compatible versions
      Adding cc v1.0.106 (available: v1.2.51)
      Adding colored v2.2.0 (available: v3.0.0)
      Adding crossterm v0.28.1 (available: v0.29.0)
      Adding generic-array v0.14.7 (available: v0.14.9)
      Adding thiserror v1.0.69 (available: v2.0.17)
      Adding tiktoken-rs v0.5.9 (available: v0.9.1)
      Adding toml v0.8.23 (available: v0.9.10+spec-1.1.0)
      Adding tree-sitter v0.20.10 (available: v0.26.3)
      Adding tree-sitter-python v0.20.4 (available: v0.25.0)
      Adding tree-sitter-rust v0.20.4 (available: v0.24.0)
      Adding tree-sitter-typescript v0.20.5 (available: v0.23.2)
  Downloaded zmij v1.0.10
  Downloaded 1 crate (21.8KiB) in 0.24s
   Compiling zmij v1.0.10
   Compiling tree-sitter v0.20.10
   Compiling syn v2.0.113
   Compiling tree-sitter-rust v0.20.4
   Compiling tree-sitter-typescript v0.20.5
   Compiling tree-sitter-python v0.20.4
   Compiling either v1.15.0
   Compiling rayon v1.11.0
   Compiling serde_json v1.0.148
   Compiling serde_derive v1.0.228
   Compiling clap_derive v4.5.49
   Compiling thiserror-impl v1.0.69
   Compiling thiserror v1.0.69
   Compiling clap v4.5.54
   Compiling serde v1.0.228
   Compiling serde_spanned v0.6.9
   Compiling toml_datetime v0.6.11
   Compiling toml_edit v0.22.27
   Compiling toml v0.8.23
   Compiling slopchop v1.3.4 (C:\Users\junov\slopchop)
    Finished `release` profile [optimized] target(s) in 1m 08s
   Replacing C:\Users\junov\.cargo\bin\slopchop.exe
    Replaced package `slopchop v1.3.4 (C:\Users\junov\slopchop)` with `slopchop v1.3.4 (C:\Users\junov\slopchop)` (executable `slopchop.exe`)
slopchop on  main [!?] is 📦 v1.3.4 via 🦀 v1.91.1 took 1m9s
❯ slopchop scan --json
error: unexpected argument '--json' found

Usage: slopchop.exe scan [OPTIONS]

For more information, try '--help'.
slopchop on  main [!?] is 📦 v1.3.4 via 🦀 v1.91.1
❯ cargo install --path . --force
  Installing slopchop v1.3.4 (C:\Users\junov\slopchop)
    Updating crates.io index
     Locking 108 packages to latest compatible versions
      Adding cc v1.0.106 (available: v1.2.51)
      Adding colored v2.2.0 (available: v3.0.0)
      Adding crossterm v0.28.1 (available: v0.29.0)
      Adding generic-array v0.14.7 (available: v0.14.9)
      Adding thiserror v1.0.69 (available: v2.0.17)
      Adding tiktoken-rs v0.5.9 (available: v0.9.1)
      Adding toml v0.8.23 (available: v0.9.10+spec-1.1.0)
      Adding tree-sitter v0.20.10 (available: v0.26.3)
      Adding tree-sitter-python v0.20.4 (available: v0.25.0)
      Adding tree-sitter-rust v0.20.4 (available: v0.24.0)
      Adding tree-sitter-typescript v0.20.5 (available: v0.23.2)
   Compiling slopchop v1.3.4 (C:\Users\junov\slopchop)
    Finished `release` profile [optimized] target(s) in 1m 01s
   Replacing C:\Users\junov\.cargo\bin\slopchop.exe
    Replaced package `slopchop v1.3.4 (C:\Users\junov\slopchop)` with `slopchop v1.3.4 (C:\Users\junov\slopchop)` (executable `slopchop.exe`)
slopchop on  main [!?] is 📦 v1.3.4 via 🦀 v1.91.1 took 1m1s
❯ slopchop scan --json
{
  "files": [
    {
      "path": "Cargo.toml",
      "token_count": 526,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "Changelog.md",
      "token_count": 1292,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "docs\\v1.3.5\\past-present-future.md",
      "token_count": 581,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "docs\\v1.3.5\\v1.3.5-brief.md",
      "token_count": 799,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "README.md",
      "token_count": 2189,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "slopchop.toml",
      "token_count": 364,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\analysis\\ast.rs",
      "token_count": 756,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\analysis\\checks\\banned.rs",
      "token_count": 529,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\analysis\\checks\\complexity.rs",
      "token_count": 1540,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\analysis\\checks\\naming.rs",
      "token_count": 731,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\analysis\\checks.rs",
      "token_count": 117,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\analysis\\metrics.rs",
      "token_count": 462,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\analysis\\mod.rs",
      "token_count": 1281,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\analysis\\sabotage.rs",
      "token_count": 1044,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\analysis\\safety.rs",
      "token_count": 930,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\backup.rs",
      "token_count": 973,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\blocks.rs",
      "token_count": 1038,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\executor.rs",
      "token_count": 1590,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\manifest.rs",
      "token_count": 420,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\messages.rs",
      "token_count": 796,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\mod.rs",
      "token_count": 523,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\parser.rs",
      "token_count": 1492,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\patch\\common.rs",
      "token_count": 890,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\patch\\diagnostics.rs",
      "token_count": 1499,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\patch\\parser_v0.rs",
      "token_count": 427,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\patch\\parser_v1.rs",
      "token_count": 937,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\patch.rs",
      "token_count": 1275,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\processor.rs",
      "token_count": 1684,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\process_runner.rs",
      "token_count": 732,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\types.rs",
      "token_count": 511,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\validator.rs",
      "token_count": 1601,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\verification.rs",
      "token_count": 1572,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\apply\\writer.rs",
      "token_count": 1239,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\clean.rs",
      "token_count": 649,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\cli\\args.rs",
      "token_count": 829,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\cli\\audit.rs",
      "token_count": 751,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\cli\\config_ui\\editor.rs",
      "token_count": 1592,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\cli\\config_ui\\items.rs",
      "token_count": 655,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\cli\\config_ui\\mod.rs",
      "token_count": 28,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\cli\\config_ui\\render.rs",
      "token_count": 485,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\cli\\handlers.rs",
      "token_count": 1835,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\cli\\locality.rs",
      "token_count": 806,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\cli\\mod.rs",
      "token_count": 78,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\clipboard\\linux.rs",
      "token_count": 918,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\clipboard\\macos.rs",
      "token_count": 264,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\clipboard\\mod.rs",
      "token_count": 400,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\clipboard\\platform.rs",
      "token_count": 87,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\clipboard\\temp.rs",
      "token_count": 396,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\clipboard\\utils.rs",
      "token_count": 201,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\clipboard\\windows.rs",
      "token_count": 294,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\config\\io.rs",
      "token_count": 925,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\config\\locality.rs",
      "token_count": 560,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\config\\mod.rs",
      "token_count": 453,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\config\\types.rs",
      "token_count": 1014,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\constants.rs",
      "token_count": 451,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\detection.rs",
      "token_count": 548,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\discovery.rs",
      "token_count": 962,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\error.rs",
      "token_count": 73,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\events.rs",
      "token_count": 563,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\exit.rs",
      "token_count": 455,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\lang.rs",
      "token_count": 1480,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\lib.rs",
      "token_count": 105,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\pack\\docs.rs",
      "token_count": 669,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\pack\\focus.rs",
      "token_count": 660,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\pack\\formats.rs",
      "token_count": 1824,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\pack\\mod.rs",
      "token_count": 1741,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\project.rs",
      "token_count": 961,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\prompt.rs",
      "token_count": 991,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\reporting.rs",
      "token_count": 821,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\skeleton.rs",
      "token_count": 766,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\spinner.rs",
      "token_count": 435,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\stage\\copy.rs",
      "token_count": 1135,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\stage\\manager.rs",
      "token_count": 1787,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\stage\\mod.rs",
      "token_count": 618,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\stage\\promote.rs",
      "token_count": 1673,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\stage\\state.rs",
      "token_count": 1794,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\tokens.rs",
      "token_count": 302,
      "complexity_score": 0,
      "violations": []
    },
    {
      "path": "src\\types.rs",
      "token_count": 618,
      "complexity_score": 0,
      "violations": []
    }
  ],
  "total_tokens": 66992,
  "total_violations": 0,
  "duration_ms": 186
}
slopchop on  main [!?] is 📦 v1.3.4 via 🦀 v1.91.1
❯ slopchop check --json
{
  "scan": {
    "files": [
      {
        "path": "Cargo.toml",
        "token_count": 526,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "Changelog.md",
        "token_count": 1292,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "docs\\v1.3.5\\past-present-future.md",
        "token_count": 581,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "docs\\v1.3.5\\v1.3.5-brief.md",
        "token_count": 799,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "README.md",
        "token_count": 2189,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "slopchop.toml",
        "token_count": 364,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\analysis\\ast.rs",
        "token_count": 756,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\analysis\\checks\\banned.rs",
        "token_count": 529,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\analysis\\checks\\complexity.rs",
        "token_count": 1540,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\analysis\\checks\\naming.rs",
        "token_count": 731,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\analysis\\checks.rs",
        "token_count": 117,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\analysis\\metrics.rs",
        "token_count": 462,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\analysis\\mod.rs",
        "token_count": 1281,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\analysis\\sabotage.rs",
        "token_count": 1044,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\analysis\\safety.rs",
        "token_count": 930,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\backup.rs",
        "token_count": 973,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\blocks.rs",
        "token_count": 1038,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\executor.rs",
        "token_count": 1590,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\manifest.rs",
        "token_count": 420,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\messages.rs",
        "token_count": 796,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\mod.rs",
        "token_count": 523,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\parser.rs",
        "token_count": 1492,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\patch\\common.rs",
        "token_count": 890,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\patch\\diagnostics.rs",
        "token_count": 1499,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\patch\\parser_v0.rs",
        "token_count": 427,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\patch\\parser_v1.rs",
        "token_count": 937,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\patch.rs",
        "token_count": 1275,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\processor.rs",
        "token_count": 1684,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\process_runner.rs",
        "token_count": 732,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\types.rs",
        "token_count": 511,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\validator.rs",
        "token_count": 1601,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\verification.rs",
        "token_count": 1572,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\apply\\writer.rs",
        "token_count": 1239,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\clean.rs",
        "token_count": 649,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\cli\\args.rs",
        "token_count": 829,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\cli\\audit.rs",
        "token_count": 751,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\cli\\config_ui\\editor.rs",
        "token_count": 1592,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\cli\\config_ui\\items.rs",
        "token_count": 655,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\cli\\config_ui\\mod.rs",
        "token_count": 28,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\cli\\config_ui\\render.rs",
        "token_count": 485,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\cli\\handlers.rs",
        "token_count": 1835,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\cli\\locality.rs",
        "token_count": 806,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\cli\\mod.rs",
        "token_count": 78,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\clipboard\\linux.rs",
        "token_count": 918,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\clipboard\\macos.rs",
        "token_count": 264,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\clipboard\\mod.rs",
        "token_count": 400,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\clipboard\\platform.rs",
        "token_count": 87,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\clipboard\\temp.rs",
        "token_count": 396,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\clipboard\\utils.rs",
        "token_count": 201,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\clipboard\\windows.rs",
        "token_count": 294,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\config\\io.rs",
        "token_count": 925,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\config\\locality.rs",
        "token_count": 560,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\config\\mod.rs",
        "token_count": 453,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\config\\types.rs",
        "token_count": 1014,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\constants.rs",
        "token_count": 451,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\detection.rs",
        "token_count": 548,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\discovery.rs",
        "token_count": 962,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\error.rs",
        "token_count": 73,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\events.rs",
        "token_count": 563,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\exit.rs",
        "token_count": 455,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\lang.rs",
        "token_count": 1480,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\lib.rs",
        "token_count": 105,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\pack\\docs.rs",
        "token_count": 669,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\pack\\focus.rs",
        "token_count": 660,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\pack\\formats.rs",
        "token_count": 1824,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\pack\\mod.rs",
        "token_count": 1741,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\project.rs",
        "token_count": 961,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\prompt.rs",
        "token_count": 991,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\reporting.rs",
        "token_count": 821,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\skeleton.rs",
        "token_count": 766,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\spinner.rs",
        "token_count": 435,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\stage\\copy.rs",
        "token_count": 1135,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\stage\\manager.rs",
        "token_count": 1787,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\stage\\mod.rs",
        "token_count": 618,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\stage\\promote.rs",
        "token_count": 1673,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\stage\\state.rs",
        "token_count": 1794,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\tokens.rs",
        "token_count": 302,
        "complexity_score": 0,
        "violations": []
      },
      {
        "path": "src\\types.rs",
        "token_count": 618,
        "complexity_score": 0,
        "violations": []
      }
    ],
    "total_tokens": 66992,
    "total_violations": 0,
    "duration_ms": 184
  },
  "commands": [
    {
      "command": "cargo clippy --all-targets -- -D warnings -W clippy::pedantic",
      "exit_code": 0,
      "stdout": "",
      "stderr": "   Compiling tree-sitter v0.20.10\n   Compiling tree-sitter-rust v0.20.4\n   Compiling tree-sitter-python v0.20.4\n   Compiling tree-sitter-typescript v0.20.5\n    Checking slopchop v1.3.4 (C:\\Users\\junov\\slopchop)\n    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.18s\n",
      "duration_ms": 4369
    },
    {
      "command": "slopchop scan --locality",
      "exit_code": 0,
      "stdout": "",
      "stderr": "",
      "duration_ms": 0
    }
  ],
  "passed": true
}
slopchop on  main [!?] is 📦 v1.3.4 via 🦀 v1.91.1 took 4s
❯
