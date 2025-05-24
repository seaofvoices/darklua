use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::rules::convert_require::rojo_sourcemap::RojoSourcemap;
use crate::DarkluaError;
use crate::Resources;

type SourcemapCacheKey = PathBuf;
type SourcemapCacheValue = Arc<RojoSourcemap>;
type SourcemapCacheMap = HashMap<SourcemapCacheKey, SourcemapCacheValue>;

// A global cache for parsed sourcemaps
lazy_static::lazy_static! {
    static ref SOURCEMAP_CACHE: Mutex<SourcemapCacheMap> = Mutex::new(HashMap::new());
}

/// Get a sourcemap from the cache or parse it if not present
pub(crate) fn get_sourcemap(
    path: &Path,
    resources: &Resources,
    relative_to: &Path,
) -> Result<Arc<RojoSourcemap>, DarkluaError> {
    let cache_key = path.to_path_buf();

    // Try to get from cache first
    {
        let cache = SOURCEMAP_CACHE.lock().unwrap();
        if let Some(sourcemap) = cache.get(&cache_key) {
            log::debug!("Using cached sourcemap for {}", path.display());
            return Ok(Arc::clone(sourcemap));
        }
    }

    // Not in cache, load and parse the sourcemap
    log::debug!("Parsing sourcemap from {}", path.display());
    let sourcemap_timer = crate::utils::Timer::now();

    // Read the content of the sourcemap
    let content = resources.get(path)?;

    // Check if content is empty or invalid
    if content.trim().is_empty() {
        return Err(DarkluaError::custom(format!(
            "Sourcemap file '{}' is empty (unable to access or parse Rojo sourcemap at '{}')",
            path.display(),
            path.display()
        )));
    }

    // Regular JSON parsing
    let result = RojoSourcemap::parse(&content, relative_to).map_err(|err| {
        DarkluaError::custom(format!("Invalid sourcemap '{}': {}", path.display(), err))
    })?;

    log::debug!("Parsed sourcemap in {}", sourcemap_timer.duration_label());

    // Wrap in Arc and store in cache
    let sourcemap = Arc::new(result);

    {
        let mut cache = SOURCEMAP_CACHE.lock().unwrap();
        cache.insert(cache_key, Arc::clone(&sourcemap));
    }

    Ok(sourcemap)
}

/// Clear the sourcemap cache
pub(crate) fn clear_sourcemap_cache() {
    let mut cache = SOURCEMAP_CACHE.lock().unwrap();
    cache.clear();
    log::debug!("Cleared sourcemap cache");
}
