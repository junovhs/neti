// tests/unit_graph.rs
//! Tests for import extraction across languages.

use slopchop_core::graph::imports;
use std::path::Path;

#[test]
fn test_rust_use_extraction() {
    let content = r"
use std::collections::HashMap;
use crate::config::Config;
use super::types::Violation;
";
    let imports = imports::extract(Path::new("src/lib.rs"), content);
    assert!(!imports.is_empty(), "Should extract Rust use statements");
    assert!(
        imports.iter().any(|i| i.contains("HashMap")),
        "Should find HashMap import"
    );
}

#[test]
fn test_rust_mod_extraction() {
    let content = r"
mod config;
mod analysis;
pub mod types;
";
    let imports = imports::extract(Path::new("src/lib.rs"), content);
    assert!(
        imports.iter().any(|i| i.contains("config")),
        "Should extract mod declarations"
    );
}

#[test]
fn test_python_import() {
    let content = r"
import os
import sys
import json
";
    let imports = imports::extract(Path::new("main.py"), content);
    assert!(!imports.is_empty(), "Should extract Python imports");
    assert!(
        imports.iter().any(|i| i.contains("os")),
        "Should find os import"
    );
}

#[test]
fn test_python_from_import() {
    let content = r"
from pathlib import Path
from typing import Optional, List
from .utils import helper
";
    let imports = imports::extract(Path::new("main.py"), content);
    assert!(
        !imports.is_empty(),
        "Should extract from...import statements"
    );
}

#[test]
fn test_ts_import() {
    let content = r"
import { useState } from 'react';
import axios from 'axios';
import * as utils from './utils';
";
    let imports = imports::extract(Path::new("app.ts"), content);
    assert!(!imports.is_empty(), "Should extract TypeScript imports");
}
