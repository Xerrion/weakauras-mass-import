//! Load auras from files, folders, clipboard, and text input.

use std::collections::HashSet;
use std::path::PathBuf;

use iced::futures::SinkExt;
use iced::{stream, Task};
use iced_toasts::{toast, ToastLevel};

use crate::saved_variables::SavedVariablesManager;

use super::super::state::LoadingUpdate;
use super::super::{Message, WeakAuraImporter};
use super::{collect_existing_ids, decode_auras_filtered, notify_decode_results};

impl WeakAuraImporter {
    /// Scan for SavedVariables files (synchronous, called during init)
    pub(crate) fn scan_saved_variables_sync(&mut self) {
        let path = PathBuf::from(&self.wow_path);
        if path.exists() {
            self.discovered_sv_files = SavedVariablesManager::find_saved_variables(&path);
            if !self.discovered_sv_files.is_empty() {
                self.toasts.push(
                    toast(&format!(
                        "Found {} SavedVariables file(s)",
                        self.discovered_sv_files.len()
                    ))
                    .level(ToastLevel::Info),
                );
            }
        }
    }

    /// Parse the input text for WeakAura strings (appends to existing list, skips duplicates)
    pub(crate) fn parse_input(&mut self) {
        let existing_ids = collect_existing_ids(&self.parsed_auras);

        let (new_entries, added, duplicates, errors) =
            decode_auras_filtered(&self.input_text, &existing_ids);

        self.parsed_auras.extend(new_entries);
        self.selected_aura_index = None;

        notify_decode_results(&mut self.toasts, added, duplicates, &errors, "input");
    }

    /// Paste from clipboard
    pub(crate) fn paste_from_clipboard(&mut self) {
        if let Some(clipboard) = &mut self.clipboard {
            match clipboard.get_text() {
                Ok(text) => {
                    self.input_text = text;
                    self.parse_input();
                }
                Err(e) => {
                    self.toasts.push(
                        toast(&format!("Clipboard error: {}", e))
                            .title("Clipboard Error")
                            .level(ToastLevel::Error),
                    );
                }
            }
        }
    }

    /// Load from file dialog (async)
    pub(crate) fn load_from_file_async(&mut self) -> Task<Message> {
        Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Text files", &["txt", "md"])
                    .add_filter("All files", &["*"])
                    .pick_file()
                    .await
                    .map(|h| h.path().to_path_buf())
            },
            Message::FileSelected,
        )
    }

    /// Load file content after selection (async)
    pub(crate) fn load_file_content_async(&mut self, path: PathBuf) -> Task<Message> {
        let existing_ids = collect_existing_ids(&self.parsed_auras);

        self.is_loading = true;
        self.loading_progress = 0.0;
        self.loading_message = format!("Loading {}...", path.display());

        Task::perform(
            async move {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        let (entries, added, duplicates, errors) =
                            decode_auras_filtered(&content, &existing_ids);

                        LoadingUpdate::Complete {
                            entries,
                            added,
                            duplicates,
                            errors,
                        }
                    }
                    Err(e) => LoadingUpdate::Error(format!("Failed to read file: {}", e)),
                }
            },
            Message::LoadingUpdate,
        )
    }

    /// Load from folder dialog (async)
    pub(crate) fn load_from_folder_async(&mut self) -> Task<Message> {
        Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .pick_folder()
                    .await
                    .map(|h| h.path().to_path_buf())
            },
            Message::FolderSelected,
        )
    }

    /// Load folder content after selection (async)
    pub(crate) fn load_folder_content_async(&mut self, folder_path: PathBuf) -> Task<Message> {
        // Scan folder synchronously (fast filesystem walk)
        let file_paths = match scan_folder_recursive(&folder_path) {
            Ok(paths) => paths,
            Err(e) => {
                self.toasts.push(
                    toast(&format!("Failed to scan folder: {}", e))
                        .title("Folder Error")
                        .level(ToastLevel::Error),
                );
                return Task::none();
            }
        };

        if file_paths.is_empty() {
            self.toasts
                .push(toast("No supported files found in folder").level(ToastLevel::Warning));
            return Task::none();
        }

        let existing_ids = collect_existing_ids(&self.parsed_auras);

        self.is_loading = true;
        self.loading_progress = 0.0;
        self.loading_message = format!("Processing {} file(s)...", file_paths.len());

        let total_files = file_paths.len();

        Task::run(
            stream::channel(
                100,
                move |mut sender: iced::futures::channel::mpsc::Sender<Message>| async move {
                    process_folder_files(file_paths, total_files, existing_ids, &mut sender).await;
                },
            ),
            |msg| msg,
        )
    }
}

/// Recursively scan a folder for supported files (.txt, .md, .lua)
fn scan_folder_recursive(folder: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let supported_extensions = ["txt", "md", "lua"];

    fn visit_dir(
        dir: &PathBuf,
        files: &mut Vec<PathBuf>,
        extensions: &[&str],
    ) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, files, extensions)?;
                } else if let Some(ext) = path.extension() {
                    if extensions.iter().any(|e| ext.eq_ignore_ascii_case(e)) {
                        files.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    visit_dir(folder, &mut files, &supported_extensions)?;
    Ok(files)
}

/// Process multiple files from a folder with progress updates
async fn process_folder_files(
    file_paths: Vec<PathBuf>,
    total_files: usize,
    existing_ids: HashSet<String>,
    sender: &mut iced::futures::channel::mpsc::Sender<Message>,
) {
    let mut all_entries = Vec::new();
    let mut total_added = 0;
    let mut total_duplicates = 0;
    let mut all_errors = Vec::new();
    let mut batch_ids = existing_ids;

    for (idx, file_path) in file_paths.iter().enumerate() {
        let current = idx + 1;
        let _ = sender
            .send(Message::LoadingUpdate(LoadingUpdate::Progress {
                current,
                total: total_files,
                message: format!("Processing file {} of {}...", current, total_files),
            }))
            .await;

        let content = match tokio::fs::read_to_string(&file_path).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (entries, added, duplicates, errors) = decode_auras_filtered(&content, &batch_ids);

        // Add newly discovered IDs to batch set for cross-file dedup
        for entry in &entries {
            if let Some(ref id) = entry.validation.aura_id {
                batch_ids.insert(id.clone());
            }
        }

        all_entries.extend(entries);
        total_added += added;
        total_duplicates += duplicates;
        all_errors.extend(errors);
    }

    let _ = sender
        .send(Message::LoadingUpdate(LoadingUpdate::Complete {
            entries: all_entries,
            added: total_added,
            duplicates: total_duplicates,
            errors: all_errors,
        }))
        .await;
}
