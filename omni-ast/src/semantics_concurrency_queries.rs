use super::{SemanticContext, SemanticLanguage};

pub(super) fn is_async_locking_context(
    language: SemanticLanguage,
    context: &SemanticContext,
) -> bool {
    let source = context.source_text.to_lowercase();

    match language {
        SemanticLanguage::Rust => {
            contains_any(
                &source,
                &[
                    ".lock().await",
                    ".read().await",
                    ".write().await",
                    "tokio::sync::mutex",
                    "tokio::sync::rwlock",
                    "tokio::sync::semaphore",
                    "futures::lock::mutex",
                    "futures_util::lock::mutex",
                    "async_std::sync::mutex",
                    "async_lock::mutex",
                ],
            )
        }
        SemanticLanguage::Python => contains_any(&source, &["asyncio.lock", "async with"]),
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => {
            contains_any(&source, &["await mutex", "await lock", "async-lock"])
        }
        SemanticLanguage::Go | SemanticLanguage::Cpp | SemanticLanguage::Swift => false,
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
