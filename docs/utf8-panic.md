# Fix: UTF-8 Safe String Truncation

## File: src/apply/verification.rs

**Find this pattern (around line 104):**
```rust
let truncated = if combined.len() > 1000 {
    format!("{}...\n[truncated]", &combined[..1000])
} else {
    combined.clone()
};
```

**Replace with:**
```rust
let truncated = if combined.len() > 1000 {
    let safe_end = floor_char_boundary(&combined, 1000);
    format!("{}...\n[truncated]", &combined[..safe_end])
} else {
    combined.clone()
};
```

**Add this helper function (or import from existing):**
```rust
/// Finds the largest valid char boundary <= idx.
fn floor_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    while !s.is_char_boundary(idx) {
        idx = idx.saturating_sub(1);
    }
    idx
}
```

**Note:** You already have `floor_char_boundary` in `src/apply/patch/diagnostics.rs` — consider moving it to a shared `src/utils.rs` and importing it.
