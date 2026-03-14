use crate::semantics::SemanticLanguage;

pub(super) fn path_contains(path: &str, needles: &[&str]) -> bool {
    let path = path.to_lowercase();
    needles.iter().any(|needle| path.contains(&needle.to_lowercase()))
}

pub(super) fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    let haystack = haystack.to_lowercase();
    needles
        .iter()
        .any(|needle| haystack.contains(&needle.to_lowercase()))
}

pub(super) fn heap_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &[
            ".clone(",
            ".to_string(",
            ".to_owned(",
            "vec![",
            "string::from",
            "format!(",
            "to_vec(",
        ],
        SemanticLanguage::Python => &[
            ".copy(",
            "list(",
            "dict(",
            "set(",
            "str(",
            "bytes(",
            "copy.deepcopy",
        ],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => &[
            ".slice(",
            ".map(",
            ".filter(",
            ".concat(",
            "array.from(",
            "new map(",
            "new set(",
        ],
        SemanticLanguage::Go => &["append(", "make([]", "make(map[", "string("],
        SemanticLanguage::Cpp => &["std::string", "std::vector", "std::map", "std::set"],
        SemanticLanguage::Swift => &["array(", "dictionary(", "set(", "string("],
    }
}

pub(super) fn heap_type_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &[
            "string", "vec", "hashmap", "btreemap", "hashset", "bytes", "name", "text", "data",
            "list", "array", "items", "cache",
        ],
        SemanticLanguage::Python => &["list", "dict", "set", "str", "bytes", "items", "cache"],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => {
            &["string", "array", "map", "set", "record", "items", "cache"]
        }
        SemanticLanguage::Go => &["string", "[]", "map["],
        SemanticLanguage::Cpp => &["std::string", "std::vector", "std::map", "std::set"],
        SemanticLanguage::Swift => &["string", "array", "dictionary", "set"],
    }
}

pub(super) fn lookup_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &[
            ".find(",
            ".position(",
            ".contains(",
            ".contains_key(",
            ".get(",
            ".binary_search(",
        ],
        SemanticLanguage::Python => &[" in ", ".index(", ".get(", ".count("],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => {
            &[".find(", ".findindex(", ".includes(", ".indexof(", ".get(", ".has("]
        }
        SemanticLanguage::Go => &["strings.contains(", "slices.contains(", "maps.lookup", "map["],
        SemanticLanguage::Cpp => &[".find(", ".contains(", "std::find("],
        SemanticLanguage::Swift => &[".contains(", ".firstindex(", ".first(where:"],
    }
}

pub(super) fn lookup_import_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &[],
        SemanticLanguage::Python => &["bisect", "collections"],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => &[],
        SemanticLanguage::Go => &["slices", "strings"],
        SemanticLanguage::Cpp => &["algorithm", "unordered_map", "map"],
        SemanticLanguage::Swift => &[],
    }
}

pub(super) fn length_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &[".len()", ".is_empty()"],
        SemanticLanguage::Python => &["len(", ".__len__("],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => &[".length", ".size"],
        SemanticLanguage::Go => &["len("],
        SemanticLanguage::Cpp => &[".size()", ".empty()"],
        SemanticLanguage::Swift => &[".count", ".isempty"],
    }
}

pub(super) fn mutation_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &[
            "push(",
            "insert(",
            "remove(",
            "clear(",
            "append(",
            "mut self",
            "&mut self",
        ],
        SemanticLanguage::Python => &["append(", "extend(", "update(", "pop(", "remove("],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => {
            &["push(", "splice(", "set(", "delete ", ".assign("]
        }
        SemanticLanguage::Go => &["append(", "delete("],
        SemanticLanguage::Cpp => &["push_back(", "insert(", "erase(", "clear("],
        SemanticLanguage::Swift => &["append(", "insert(", "remove(", "removeall("],
    }
}

pub(super) fn locking_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &["mutex", "rwlock", ".lock()", ".read()", ".write()"],
        SemanticLanguage::Python => &["threading.lock", "asyncio.lock", ".acquire("],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => &["atomics.", "mutex", "lock("],
        SemanticLanguage::Go => &["sync.mutex", "sync.rwmutex", ".lock()", ".rlock()"],
        SemanticLanguage::Cpp => &["std::mutex", "std::lock_guard", ".lock()"],
        SemanticLanguage::Swift => &["nslock", "dispatchqueue", ".lock()"],
    }
}

pub(super) fn locking_import_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &["std::sync"],
        SemanticLanguage::Python => &["threading", "asyncio"],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => &["worker_threads"],
        SemanticLanguage::Go => &["sync"],
        SemanticLanguage::Cpp => &["mutex"],
        SemanticLanguage::Swift => &["foundation", "dispatch"],
    }
}

pub(super) fn loop_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &["for ", "while ", "loop {"],
        SemanticLanguage::Python => &["for ", "while "],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => {
            &["for (", "while (", "for await", ".foreach("]
        }
        SemanticLanguage::Go => &["for "],
        SemanticLanguage::Cpp => &["for (", "while ("],
        SemanticLanguage::Swift => &["for ", "while "],
    }
}

pub(super) fn exported_api_needles(language: SemanticLanguage) -> &'static [&'static str] {
    match language {
        SemanticLanguage::Rust => &["pub fn ", "pub struct ", "pub enum ", "pub trait "],
        SemanticLanguage::Python => &["__all__", "class ", "def "],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => {
            &["export function", "export class", "export const", "module.exports"]
        }
        SemanticLanguage::Go => &["func ", "type "],
        SemanticLanguage::Cpp => &["public:", "class ", "struct "],
        SemanticLanguage::Swift => &["public func", "public struct", "public class"],
    }
}
