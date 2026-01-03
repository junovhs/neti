# Hash Flip-Flopping Investigation

**Status:** Resolved  
**Discovered:** 2026-01-02  
**Resolved:** 2026-01-02  

---

## Root Cause

Two separate hash functions existed without line ending normalization:
- `src/pack/formats.rs`  `compute_hash()` 
- `src/apply/patch/common.rs`  `compute_sha256()`

On Windows with CRLF files, the same file content could hash differently.

---

## Fix (v1.3.3)

1. `compute_sha256()` now normalizes CRLF/CR  LF before hashing
2. Removed duplicate `compute_hash()`, single source of truth in `common.rs`
3. Added `test_eol_normalization` and `test_hash_stability` tests

---

## Verification

```
$ printf 'fn main() {\r\n    println!("test");\r\n}\r\n' > test_crlf.rs
$ printf 'fn main() {\n    println!("test");\n}\n' > test_lf.rs

$ slopchop pack --focus test_crlf.rs --noprompt && grep SHA256 context.txt
SHA256:eea490a95676a2c4dc5d196c8d29984f084ed1e641bccaab56a8cf659b7e3ddb

$ slopchop pack --focus test_lf.rs --noprompt && grep SHA256 context.txt
SHA256:eea490a95676a2c4dc5d196c8d29984f084ed1e641bccaab56a8cf659b7e3ddb
```

Identical hashes confirm fix works.