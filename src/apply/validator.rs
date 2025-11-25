// src/apply/validator.rs
use crate::apply::messages;
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, Operation};

#[must_use]
pub fn validate(manifest: &Manifest, extracted: &ExtractedFiles) -> ApplyOutcome {
    let mut missing = Vec::new();
    let mut errors = Vec::new();

    // 1. Check for Missing Files (Declared in Manifest but not provided)
    for entry in manifest {
        if entry.operation != Operation::Delete && !extracted.contains_key(&entry.path) {
            missing.push(entry.path.clone());
        }
    }

    // 2. Check for Extra Files (Optional - maybe we allow this?)
    // For now, we accept extra files but maybe warn. Warden is strict, but "Bonus code" is okay.

    // 3. Content Validation
    for (path, file) in extracted {
        if file.content.trim().is_empty() {
            errors.push(format!("{path} is empty"));
        }
        // Future: Check for truncation markers like "// ..."
    }

    if !missing.is_empty() || !errors.is_empty() {
        let ai_message = messages::format_ai_rejection(&missing, &errors);
        return ApplyOutcome::ValidationFailure {
            errors,
            missing,
            ai_message,
        };
    }

    // Success - list what we are about to write
    let written = extracted.keys().cloned().collect();
    ApplyOutcome::Success {
        written,
        backed_up: true,
    }
}
