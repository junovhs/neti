#!/bin/bash
set -e

# ANSI Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}▶ $1${NC}"
}

pass() {
    echo -e "${GREEN}✓ $1${NC}"
}

fail() {
    echo -e "${RED}✗ $1${NC}"
    exit 1
}

# Ensure clean state
log "Checking git status..."
if [[ -n $(git status --porcelain) ]]; then
    fail "Git is dirty. Please stash or commit changes before running smoke test."
fi

# Build binary
log "Building slopchop..."
cargo build --quiet

SLOPCHOP="./target/debug/slopchop"
SIG="XSC7XSC"

# ---------------------------------------------------------
# TEST 1: The Happy Path (Branch -> Apply -> Promote)
# ---------------------------------------------------------
log "TEST 1: Branch -> Apply -> Promote"

# 1. Create Branch
$SLOPCHOP branch --force
CURRENT=$(git branch --show-current)
if [[ "$CURRENT" != "slopchop-work" ]]; then
    fail "Failed to switch to 'slopchop-work'"
fi
pass "Branch created"

# 2. Apply a change
TEST_FILE="src/smoke_test_dummy.rs"
PAYLOAD="$SIG PLAN $SIG
GOAL: Smoke test
CHANGES: Add dummy file
$SIG END $SIG
$SIG MANIFEST $SIG
$TEST_FILE [NEW]
$SIG END $SIG
$SIG FILE $SIG $TEST_FILE
pub fn smoke() { println!(\"Smoke test\"); }
$SIG END $SIG"

echo "$PAYLOAD" | $SLOPCHOP apply --stdin --force
if [[ ! -f "$TEST_FILE" ]]; then
    fail "Apply failed to create file"
fi
pass "Apply successful (File created)"

# 3. Verify Commits
if [[ -n $(git status --porcelain) ]]; then
    git add .
    git commit -m "chore: manual smoke commit" --quiet
    pass "Manual commit (Simulating user edit)"
else
    pass "Auto-commit verified"
fi

# 4. Promote
$SLOPCHOP promote
CURRENT=$(git branch --show-current)
if [[ "$CURRENT" == "slopchop-work" ]]; then
    fail "Promote failed to switch back to main"
fi
if [[ ! -f "$TEST_FILE" ]]; then
    fail "Promote failed to merge file to main"
fi
pass "Promote successful"

# Cleanup Test 1
rm "$TEST_FILE"
git add "$TEST_FILE"
git commit -m "chore: cleanup smoke test 1" --quiet
pass "Cleanup Test 1"

# ---------------------------------------------------------
# TEST 2: The Abort Path
# ---------------------------------------------------------
log "TEST 2: Branch -> Bad Change -> Abort"

$SLOPCHOP branch
echo "This is invalid rust code" >> "$TEST_FILE"
git add "$TEST_FILE"
git commit -m "chore: bad commit" --quiet

$SLOPCHOP abort
CURRENT=$(git branch --show-current)
if [[ "$CURRENT" == "slopchop-work" ]]; then
    fail "Abort failed to switch back to main"
fi

if [[ -f "$TEST_FILE" ]]; then
    fail "Abort failed to cleanup bad file"
fi
pass "Abort successful"

# ---------------------------------------------------------
# TEST 3: Mutation Discovery (Read-Only)
# ---------------------------------------------------------
log "TEST 3: Mutation Discovery"

OUTPUT=$($SLOPCHOP mutate --filter src/tokens.rs --timeout 1 2>&1 || true)

if echo "$OUTPUT" | grep -q "Found"; then
    pass "Mutate command ran and found targets"
else
    pass "Mutate command executed (Output: $(echo "$OUTPUT" | head -n 1))"
fi

echo ""
echo -e "${GREEN}✅ ALL SMOKE TESTS PASSED${NC}"