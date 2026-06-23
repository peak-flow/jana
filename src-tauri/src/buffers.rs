use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};

use crate::frontmatter;

/// The content authority for one open file. There is exactly one `BufferState` per
/// open file, shared by every tab/window viewing it. The backend owns the content
/// and is the only writer to disk.
pub struct BufferState {
    pub file_path: String,
    pub jana_id: String,
    pub content: String,
    pub version: u64,
    pub last_saved_version: u64,
    pub refcount: u32,
    /// Unix millis of the last accepted edit — drives the flush loop's idle check.
    pub last_edit: i64,
}

impl BufferState {
    fn dirty(&self) -> bool {
        self.version > self.last_saved_version
    }
}

#[derive(Clone, Default)]
pub struct BufferRegistry {
    inner: Arc<Mutex<HashMap<String, BufferState>>>,
}

impl BufferRegistry {
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, BufferState>> {
        self.inner.lock().expect("buffer registry mutex poisoned")
    }
}

#[derive(Serialize)]
pub struct AcquireResult {
    pub buffer_id: String,
    pub jana_id: String,
    pub content: String,
    pub version: u64,
    pub file_name: String,
}

#[derive(Serialize)]
pub struct BufferSnapshot {
    pub jana_id: String,
    pub content: String,
    pub version: u64,
}

#[derive(Serialize)]
pub struct UpdateResult {
    pub version: u64,
    pub conflict: bool,
    /// Authoritative content, sent back only on conflict so the caller can resync.
    pub content: Option<String>,
}

/// Broadcast to every window when a buffer accepts an edit. Peers apply `changes`
/// (a CodeMirror ChangeSet, opaque to the backend) to their own view; the window
/// that originated the edit filters its own event out via `origin_window_id`.
#[derive(Clone, Serialize)]
pub struct BufferUpdatedEvent {
    pub buffer_id: String,
    pub changes: serde_json::Value,
    pub version: u64,
    pub origin_window_id: String,
}

fn now_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn file_name_of(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Canonical absolute path = the open-file identity. Resolving symlinks/`..` means
/// two paths to the same file map to one buffer. Falls back to the raw path when
/// the file can't be canonicalized (should not happen for files we have opened).
pub fn canonical_buffer_id(file_path: &str) -> String {
    std::fs::canonicalize(file_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| file_path.to_string())
}

/// Register a view on a file's buffer, loading it from disk on first reference.
#[tauri::command]
pub async fn acquire_buffer(
    file_path: String,
    registry: State<'_, BufferRegistry>,
) -> Result<AcquireResult, String> {
    let buffer_id = canonical_buffer_id(&file_path);
    let file_name = file_name_of(&file_path);

    // Fast path: already loaded — just add a reference.
    {
        let mut map = registry.lock();
        if let Some(buf) = map.get_mut(&buffer_id) {
            buf.refcount += 1;
            return Ok(AcquireResult {
                buffer_id,
                jana_id: buf.jana_id.clone(),
                content: buf.content.clone(),
                version: buf.version,
                file_name,
            });
        }
    }

    // Load from disk outside the lock (this may inject a jana_id and write).
    let (jana_id, content) = frontmatter::ensure_jana_id(&file_path)?;

    let mut map = registry.lock();
    let buf = map.entry(buffer_id.clone()).or_insert_with(|| BufferState {
        file_path: file_path.clone(),
        jana_id: jana_id.clone(),
        content: content.clone(),
        version: 0,
        last_saved_version: 0,
        refcount: 0,
        last_edit: now_millis(),
    });
    buf.refcount += 1;
    Ok(AcquireResult {
        buffer_id,
        jana_id: buf.jana_id.clone(),
        content: buf.content.clone(),
        version: buf.version,
        file_name,
    })
}

/// Read a buffer's current content + version without changing its refcount. Used by
/// the editor each time it loads a tab, so it always shows authoritative content.
#[tauri::command]
pub async fn get_buffer(
    buffer_id: String,
    registry: State<'_, BufferRegistry>,
) -> Result<BufferSnapshot, String> {
    let map = registry.lock();
    let buf = map.get(&buffer_id).ok_or("Buffer not found")?;
    Ok(BufferSnapshot {
        jana_id: buf.jana_id.clone(),
        content: buf.content.clone(),
        version: buf.version,
    })
}

/// Apply an edit to a buffer. Accepts the edit only if `base_version` matches the
/// current version (kills last-writer-wins); otherwise returns the authoritative
/// content so the caller can resync. On acceptance, broadcasts `buffer-updated` to
/// every window so peers viewing the same buffer apply the change live.
#[tauri::command]
pub async fn update_buffer(
    app: tauri::AppHandle,
    buffer_id: String,
    content: String,
    changes: serde_json::Value,
    base_version: u64,
    origin_window_id: String,
    registry: State<'_, BufferRegistry>,
) -> Result<UpdateResult, String> {
    // Mutate under the lock, then release it before emitting.
    let (version, conflict, content_opt) = {
        let mut map = registry.lock();
        let buf = map.get_mut(&buffer_id).ok_or("Buffer not found")?;
        if base_version == buf.version {
            buf.content = content;
            buf.version += 1;
            buf.last_edit = now_millis();
            (buf.version, false, None)
        } else {
            (buf.version, true, Some(buf.content.clone()))
        }
    };

    // Only accepted edits are broadcast; a conflicting caller resyncs from the
    // returned content instead of peers re-applying a stale change.
    if !conflict {
        if let Err(e) = app.emit(
            "buffer-updated",
            BufferUpdatedEvent {
                buffer_id,
                changes,
                version,
                origin_window_id,
            },
        ) {
            eprintln!("failed to emit buffer-updated: {}", e);
        }
    }

    Ok(UpdateResult {
        version,
        conflict,
        content: content_opt,
    })
}

/// Drop a view's reference. When the last view is released, the buffer is flushed
/// to disk (if dirty) and only then removed from the registry — a failed write
/// leaves the buffer registered so unsaved content is never discarded.
#[tauri::command]
pub async fn release_buffer(
    buffer_id: String,
    registry: State<'_, BufferRegistry>,
) -> Result<(), String> {
    release_in_registry(&registry, &buffer_id)
}

/// Refcount-aware release + flush, factored out of the command so it is unit-testable.
pub fn release_in_registry(registry: &BufferRegistry, buffer_id: &str) -> Result<(), String> {
    // Phase 1 (locked): drop the ref. If this was the last view, a clean buffer is
    // evicted immediately; a dirty one is snapshotted but left registered until its
    // write succeeds, so we never remove content that isn't safely on disk yet.
    let snapshot = {
        let mut map = registry.lock();
        match map.get_mut(buffer_id) {
            Some(buf) => {
                if buf.refcount > 0 {
                    buf.refcount -= 1;
                }
                if buf.refcount == 0 {
                    if buf.dirty() {
                        Some((
                            buf.file_path.clone(),
                            buf.jana_id.clone(),
                            buf.content.clone(),
                            buf.version,
                        ))
                    } else {
                        map.remove(buffer_id);
                        None
                    }
                } else {
                    None
                }
            }
            None => None,
        }
    };

    let Some((file_path, jana_id, content, version)) = snapshot else {
        return Ok(());
    };

    // Phase 2 (unlocked): write the final content. On failure the buffer stays
    // registered with its unsaved content and the error surfaces to the caller.
    let full = frontmatter::compose_with_frontmatter(&jana_id, &content);
    std::fs::write(&file_path, &full).map_err(|e| format!("Failed to flush buffer: {}", e))?;

    // Phase 3 (locked): the write landed. Mark what we saved, and evict only if the
    // buffer is still unreferenced and unchanged. If it was re-acquired or edited
    // mid-write, keep it — the next release/flush will persist any newer content.
    let mut map = registry.lock();
    let mut evict = false;
    if let Some(buf) = map.get_mut(buffer_id) {
        if buf.version == version {
            buf.last_saved_version = version;
            if buf.refcount == 0 {
                evict = true;
            }
        }
    }
    if evict {
        map.remove(buffer_id);
    }
    Ok(())
}

/// Update a loaded buffer's jana_id after a fork, so the flush loop writes the new
/// identity rather than re-stamping the file with the old one.
pub fn set_buffer_jana_id(registry: &BufferRegistry, file_path: &str, new_jana_id: &str) {
    let buffer_id = canonical_buffer_id(file_path);
    let mut map = registry.lock();
    if let Some(buf) = map.get_mut(&buffer_id) {
        buf.jana_id = new_jana_id.to_string();
    }
}

/// Read a loaded buffer's (jana_id, content) by file path — authoritative source
/// for operations like Save As.
pub fn buffer_content(registry: &BufferRegistry, file_path: &str) -> Option<(String, String)> {
    let buffer_id = canonical_buffer_id(file_path);
    let map = registry.lock();
    map.get(&buffer_id).map(|b| (b.jana_id.clone(), b.content.clone()))
}

/// Re-key a buffer when its file moves (Save As). Preserves refcount/version and
/// marks it clean (the caller has just written `content` to `new_path`). Returns the
/// new buffer_id + version, or None if the old buffer wasn't loaded.
pub fn rekey_buffer(
    registry: &BufferRegistry,
    old_path: &str,
    new_path: &str,
    jana_id: &str,
    content: &str,
) -> Option<(String, u64)> {
    let old_id = canonical_buffer_id(old_path);
    let new_id = canonical_buffer_id(new_path);
    let mut map = registry.lock();
    if let Some(mut buf) = map.remove(&old_id) {
        buf.file_path = new_path.to_string();
        buf.jana_id = jana_id.to_string();
        buf.content = content.to_string();
        buf.last_saved_version = buf.version; // already written to disk by the caller
        let version = buf.version;
        map.insert(new_id.clone(), buf);
        Some((new_id, version))
    } else {
        None
    }
}

fn flush_where(registry: &BufferRegistry, require_idle: bool) {
    let now = now_millis();
    let pending: Vec<(String, String, String, String, u64)> = {
        let map = registry.lock();
        map.iter()
            .filter(|(_, b)| b.dirty() && (!require_idle || now - b.last_edit > 800))
            .map(|(id, b)| {
                (
                    id.clone(),
                    b.file_path.clone(),
                    b.jana_id.clone(),
                    b.content.clone(),
                    b.version,
                )
            })
            .collect()
    };

    for (id, file_path, jana_id, content, version) in pending {
        let full = frontmatter::compose_with_frontmatter(&jana_id, &content);
        match std::fs::write(&file_path, &full) {
            Ok(_) => {
                // Only advance the saved marker if no newer edit landed mid-write.
                let mut map = registry.lock();
                if let Some(buf) = map.get_mut(&id) {
                    if buf.version == version {
                        buf.last_saved_version = version;
                    }
                }
            }
            Err(e) => eprintln!("flush write failed for {}: {}", file_path, e),
        }
    }
}

/// Flush buffers that are dirty and have been idle for >800ms. The periodic tick.
pub fn flush_idle_buffers(registry: &BufferRegistry) {
    flush_where(registry, true);
}

/// Flush every dirty buffer regardless of idle time — used on app exit.
pub fn flush_all(registry: &BufferRegistry) {
    flush_where(registry, false);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("jana_buftest_{}_{}", std::process::id(), name));
        p.to_string_lossy().to_string()
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_buffer(
        reg: &BufferRegistry,
        key: &str,
        path: &str,
        jana: &str,
        content: &str,
        version: u64,
        last_saved: u64,
        last_edit: i64,
    ) {
        reg.lock().insert(
            key.to_string(),
            BufferState {
                file_path: path.to_string(),
                jana_id: jana.to_string(),
                content: content.to_string(),
                version,
                last_saved_version: last_saved,
                refcount: 1,
                last_edit,
            },
        );
    }

    #[test]
    fn flush_idle_writes_dirty_buffer_and_marks_saved() {
        let path = temp_path("flush.md");
        let _ = std::fs::remove_file(&path);
        let reg = BufferRegistry::default();
        // dirty (version 3 > saved 0) and idle (last_edit at epoch).
        insert_buffer(&reg, &path, &path, "jid-1", "hello body", 3, 0, 0);

        flush_idle_buffers(&reg);

        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert!(on_disk.contains("jana_id: jid-1"), "frontmatter id written");
        assert!(on_disk.contains("hello body"), "content written");
        assert_eq!(reg.lock().get(&path).unwrap().last_saved_version, 3);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn flush_idle_skips_recently_edited_buffer() {
        let path = temp_path("recent.md");
        let _ = std::fs::remove_file(&path);
        let reg = BufferRegistry::default();
        // dirty but edited "now" → still within the idle window → not written yet.
        insert_buffer(&reg, &path, &path, "jid-2", "x", 1, 0, now_millis());

        flush_idle_buffers(&reg);

        assert!(!std::path::Path::new(&path).exists());
    }

    #[test]
    fn flush_skips_clean_buffer() {
        let path = temp_path("clean.md");
        let _ = std::fs::remove_file(&path);
        let reg = BufferRegistry::default();
        // saved_version == version → not dirty.
        insert_buffer(&reg, &path, &path, "jid-3", "x", 2, 2, 0);

        flush_idle_buffers(&reg);

        assert!(!std::path::Path::new(&path).exists());
    }

    #[test]
    fn set_jana_id_updates_loaded_buffer() {
        let path = temp_path("fork.md");
        std::fs::write(&path, "x").unwrap();
        let key = canonical_buffer_id(&path);
        let reg = BufferRegistry::default();
        insert_buffer(&reg, &key, &path, "old-jid", "x", 1, 1, 0);

        set_buffer_jana_id(&reg, &path, "new-jid");

        assert_eq!(reg.lock().get(&key).unwrap().jana_id, "new-jid");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn release_flushes_dirty_then_evicts_last_view() {
        let path = temp_path("release_flush.md");
        let _ = std::fs::remove_file(&path);
        let reg = BufferRegistry::default();
        // last view (refcount 1) and dirty (version 2 > saved 0).
        insert_buffer(&reg, &path, &path, "jid-r", "saved body", 2, 0, 0);

        release_in_registry(&reg, &path).unwrap();

        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert!(on_disk.contains("jana_id: jid-r"), "identity written");
        assert!(on_disk.contains("saved body"), "content written");
        assert!(reg.lock().get(&path).is_none(), "evicted only after a successful write");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn release_keeps_buffer_when_flush_write_fails() {
        // A path inside a directory that does not exist makes std::fs::write fail.
        let bad = "/jana-nonexistent-dir-xyz/keep_me.md";
        let key = "k-fail";
        let reg = BufferRegistry::default();
        insert_buffer(&reg, key, bad, "jid", "precious unsaved text", 1, 0, 0);

        let res = release_in_registry(&reg, key);

        assert!(res.is_err(), "write failure is surfaced");
        let map = reg.lock();
        let buf = map.get(key).expect("buffer retained after failed flush");
        assert_eq!(buf.content, "precious unsaved text", "content preserved in registry");
    }

    #[test]
    fn release_decrements_without_flushing_when_views_remain() {
        let path = temp_path("release_multi.md");
        let _ = std::fs::remove_file(&path);
        let reg = BufferRegistry::default();
        insert_buffer(&reg, &path, &path, "jid", "x", 1, 0, 0);
        reg.lock().get_mut(&path).unwrap().refcount = 2; // a second view exists

        release_in_registry(&reg, &path).unwrap();

        assert_eq!(reg.lock().get(&path).unwrap().refcount, 1, "still referenced");
        assert!(!std::path::Path::new(&path).exists(), "no write while views remain");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn release_clean_buffer_evicts_without_writing() {
        let path = temp_path("release_clean.md");
        let _ = std::fs::remove_file(&path);
        let reg = BufferRegistry::default();
        insert_buffer(&reg, &path, &path, "jid", "x", 2, 2, 0); // clean: version == saved

        release_in_registry(&reg, &path).unwrap();

        assert!(reg.lock().get(&path).is_none(), "evicted");
        assert!(!std::path::Path::new(&path).exists(), "no write for a clean buffer");
    }

    #[test]
    fn rekey_moves_buffer_and_marks_clean() {
        let old = temp_path("old.md");
        let new = temp_path("new.md");
        std::fs::write(&old, "x").unwrap();
        std::fs::write(&new, "y").unwrap();
        let old_key = canonical_buffer_id(&old);
        let reg = BufferRegistry::default();
        insert_buffer(&reg, &old_key, &old, "jid", "body", 5, 0, 0);

        let (new_key, version) = rekey_buffer(&reg, &old, &new, "jid", "body").unwrap();

        assert_eq!(version, 5);
        let map = reg.lock();
        assert!(map.get(&old_key).is_none(), "old key removed");
        let b = map.get(&new_key).unwrap();
        assert_eq!(b.file_path, new);
        assert_eq!(b.last_saved_version, 5, "marked clean after rekey");
        drop(map);
        let _ = std::fs::remove_file(&old);
        let _ = std::fs::remove_file(&new);
    }
}
