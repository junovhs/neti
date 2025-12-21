#!/bin/bash
set -e

echo "==========================================="
echo "   SLOPCHOP PIVOT VERIFICATION PROTOCOL    "
echo "==========================================="

echo ""
echo "[1] Checking Structural Integrity (Self-Scan)..."
cargo run --bin slopchop -- check

echo ""
echo "[2] Verifying Stage Lifecycle..."
echo "    (Creation, Isolation, Reset)"
cargo test --test integration_stage_lifecycle

echo ""
echo "[3] Verifying Promotion Mechanics..."
echo "    (Atomic Swap, Rollback, Cleanup)"
cargo test --test integration_stage_promote

echo ""
echo "==========================================="
echo "   ? PIVOT SUCCESSFUL: SYSTEM NOMINAL     "
echo "==========================================="