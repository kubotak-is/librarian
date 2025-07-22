use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tauri::{AppHandle, Emitter};

/// ファイル変更イベントの種類
#[derive(Debug, Clone, serde::Serialize)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// ファイル変更イベント
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileChangeEvent {
    pub repository_id: String,
    pub file_path: String,
    pub change_type: FileChangeType,
    pub timestamp: String,
}

/// ファイル監視マネージャー
pub struct FileWatcherManager {
    watchers: Arc<RwLock<HashMap<String, RecommendedWatcher>>>,
    app: AppHandle,
    sender: mpsc::UnboundedSender<FileChangeEvent>,
}

impl FileWatcherManager {
    /// 新しいファイル監視マネージャーを作成
    #[must_use] pub fn new(app: AppHandle) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<FileChangeEvent>();
        let app_clone = app.clone();

        // イベント処理用のタスクを起動
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                // フロントエンドにイベントを送信
                if let Err(e) = app_clone.emit("file-change", &event) {
                    eprintln!("Failed to emit file change event: {e}");
                }
            }
        });

        Self {
            watchers: Arc::new(RwLock::new(HashMap::new())),
            app,
            sender,
        }
    }

    /// リポジトリの監視を開始
    pub async fn watch_repository(&self, repository_id: String, repository_path: PathBuf) -> Result<(), String> {
        let agent_library_path = repository_path.join(".agent_library");
        
        if !agent_library_path.exists() {
            return Err(format!("Agent library directory not found: {}", agent_library_path.display()));
        }

        // 既存の監視があれば停止
        self.stop_watching(&repository_id).await;

        let (tx, mut rx) = mpsc::channel(1000); // チャンネルサイズを拡大
        let sender = self.sender.clone();
        let repo_id = repository_id.clone();

        // notify の watcher を作成
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                if let Ok(event) = result {
                    if let Err(e) = tx.try_send(event) {
                        eprintln!("Failed to send file event: {e}");
                    }
                }
            },
            Config::default()
                .with_poll_interval(Duration::from_millis(500)) // ポーリング間隔を短縮
                .with_compare_contents(true), // 内容比較でfalse positiveを減らす
        ).map_err(|e| format!("Failed to create watcher: {e}"))?;

        // ディレクトリの監視を開始
        watcher
            .watch(&agent_library_path, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch directory: {e}"))?;

        // イベント処理タスクを起動
        let repo_id_clone = repo_id.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Some(change_event) = Self::process_notify_event(repo_id_clone.clone(), event) {
                    if let Err(e) = sender.send(change_event) {
                        eprintln!("Failed to forward file change event: {e}");
                        break;
                    }
                }
            }
        });

        // watcher を保存
        let mut watchers = self.watchers.write().await;
        watchers.insert(repository_id, watcher);

        Ok(())
    }

    /// リポジトリの監視を停止
    pub async fn stop_watching(&self, repository_id: &str) {
        let mut watchers = self.watchers.write().await;
        if let Some(watcher) = watchers.remove(repository_id) {
            drop(watcher); // watcher を drop することで監視を停止
        }
    }

    /// すべての監視を停止
    pub async fn stop_all_watching(&self) {
        let mut watchers = self.watchers.write().await;
        watchers.clear();
    }

    /// notify の Event を `FileChangeEvent` に変換
    fn process_notify_event(repository_id: String, event: Event) -> Option<FileChangeEvent> {
        let change_type = match event.kind {
            EventKind::Create(_) => FileChangeType::Created,
            EventKind::Modify(_) => FileChangeType::Modified,
            EventKind::Remove(_) => FileChangeType::Deleted,
            EventKind::Other => return None, // その他のイベントは無視
            _ => return None,
        };

        // パスが空の場合は無視
        if event.paths.is_empty() {
            return None;
        }

        let file_path = event.paths[0].to_string_lossy().to_string();
        
        // .agent_library ディレクトリ内のファイルのみを対象とする
        if !file_path.contains(".agent_library") {
            return None;
        }

        // 一時ファイルやスワップファイルを無視
        if Self::should_ignore_file(&event.paths[0]) {
            return None;
        }

        Some(FileChangeEvent {
            repository_id,
            file_path,
            change_type,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// 無視すべきファイルかどうかを判定
    fn should_ignore_file(path: &Path) -> bool {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // 一時ファイル、スワップファイル、システムファイルを無視
            name.starts_with('.')
                || name.starts_with('~')
                || name.ends_with('~')
                || name.ends_with(".tmp")
                || name.ends_with(".swp")
                || name.ends_with(".bak")
                || name.contains(".DS_Store")
        } else {
            false
        }
    }

    /// 監視中のリポジトリ一覧を取得
    pub async fn get_watched_repositories(&self) -> Vec<String> {
        let watchers = self.watchers.read().await;
        watchers.keys().cloned().collect()
    }
}

// ファイル監視マネージャーのグローバルインスタンス
lazy_static::lazy_static! {
    static ref FILE_WATCHER_MANAGER: Arc<RwLock<Option<FileWatcherManager>>> = Arc::new(RwLock::new(None));
}

/// ファイル監視マネージャーを初期化
pub async fn initialize_file_watcher(app: AppHandle) {
    let manager = FileWatcherManager::new(app);
    let mut global_manager = FILE_WATCHER_MANAGER.write().await;
    *global_manager = Some(manager);
}

/// グローバルファイル監視マネージャーを取得
pub async fn get_file_watcher_manager() -> Option<Arc<FileWatcherManager>> {
    let manager_guard = FILE_WATCHER_MANAGER.read().await;
    manager_guard.as_ref().map(|manager| {
        // この方法でArcに包む（実際にはマネージャー自体がArcを内包）
        Arc::new(FileWatcherManager {
            watchers: manager.watchers.clone(),
            app: manager.app.clone(),
            sender: manager.sender.clone(),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_ignore_file() {
        assert!(FileWatcherManager::should_ignore_file(Path::new(".hidden")));
        assert!(FileWatcherManager::should_ignore_file(Path::new("~backup")));
        assert!(FileWatcherManager::should_ignore_file(Path::new("file.tmp")));
        assert!(FileWatcherManager::should_ignore_file(Path::new("file.swp")));
        assert!(FileWatcherManager::should_ignore_file(Path::new(".DS_Store")));
        
        assert!(!FileWatcherManager::should_ignore_file(Path::new("normal_file.md")));
        assert!(!FileWatcherManager::should_ignore_file(Path::new("agent_index.yml")));
    }

    #[test]
    fn test_file_change_event_serialization() {
        let event = FileChangeEvent {
            repository_id: "test-repo".to_string(),
            file_path: "/path/to/file.md".to_string(),
            change_type: FileChangeType::Modified,
            timestamp: "2023-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("test-repo"));
        assert!(json.contains("Modified"));
    }

    #[tokio::test]
    async fn test_process_notify_event() {
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("/test/.agent_library/test.md")],
            attrs: Default::default(),
        };

        let result = FileWatcherManager::process_notify_event("test-repo".to_string(), event);
        assert!(result.is_some());

        let change_event = result.unwrap();
        assert_eq!(change_event.repository_id, "test-repo");
        assert!(change_event.file_path.contains(".agent_library"));
        assert!(matches!(change_event.change_type, FileChangeType::Modified));
    }
}