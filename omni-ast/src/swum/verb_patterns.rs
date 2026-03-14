//! Verb pattern expansion for SWUM.

/// Map verb to sentence pattern.
#[must_use]
pub fn expand_verb_pattern(verb: &str, rest: &str) -> String {
    if rest.is_empty() {
        return format!("Implements {verb} functionality.");
    }

    match verb {
        "get" | "fetch" | "load" | "read" | "retrieve" => format!("Gets the {rest}."),
        "set" | "write" | "save" | "store" => format!("Sets the {rest}."),
        "update" | "sync" | "refresh" => format!("Updates {rest}."),
        "is" | "has" | "can" | "should" | "will" => format!("Checks if {rest}."),
        "create" | "new" | "build" | "make" | "init" => format!("Creates {rest}."),
        "delete" | "remove" | "drop" | "clear" => format!("Removes {rest}."),
        "parse" | "extract" | "decode" => format!("Parses {rest}."),
        "validate" | "check" | "verify" => format!("Validates {rest}."),
        "render" | "format" | "display" | "print" => format!("Formats {rest} for output."),
        "handle" | "process" | "run" | "exec" => format!("Processes {rest}."),
        "convert" | "transform" | "map" => format!("Converts {rest}."),
        "find" | "search" | "lookup" | "query" => format!("Finds {rest}."),
        "test" | "spec" => format!("Tests {rest}."),
        _ => format!("Implements {verb} {rest}."),
    }
}
