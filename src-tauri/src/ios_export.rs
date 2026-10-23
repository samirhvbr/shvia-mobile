//! Staging for the dialog plugin's iOS export, tested without opening a native sheet.

use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub(crate) struct Exporter {
    active: Mutex<()>,
}

struct StagedFile(PathBuf);

impl Drop for StagedFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

impl Exporter {
    pub(crate) const fn new() -> Self {
        Self {
            active: Mutex::new(()),
        }
    }

    pub(crate) fn with_staged_file<T>(
        &self,
        documents: &Path,
        name: &str,
        bytes: &[u8],
        show_dialog: impl FnOnce() -> T,
    ) -> Result<T, String> {
        // The iOS plugin has one pending picker callback. Another export must
        // not replace it while the first sheet still owns its staged file.
        let _active = self.active.try_lock().map_err(|_| {
            "Outra exportação já está em andamento. Feche o seletor de arquivos e tente novamente."
                .to_string()
        })?;
        let path = documents.join(name);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| {
                if e.kind() == ErrorKind::AlreadyExists {
                    format!("Já existe um arquivo chamado {name} no armazenamento do app; a exportação foi cancelada para preservá-lo.")
                } else {
                    e.to_string()
                }
            })?;
        // Only a file reserved by this invocation may be removed. The guard
        // also removes a partial write if writing fails before the sheet opens.
        let _staging = StagedFile(path);
        file.write_all(bytes).map_err(|e| e.to_string())?;
        drop(file);
        Ok(show_dialog())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Documents(PathBuf);

    impl Documents {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "shvia-export-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for Documents {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn an_existing_document_is_unchanged_and_the_dialog_does_not_open() {
        let documents = Documents::new();
        let original = documents.0.join("report.pdf");
        std::fs::write(&original, b"existing document").unwrap();
        let mut opened = false;
        let result =
            Exporter::new().with_staged_file(&documents.0, "report.pdf", b"new export", || {
                opened = true;
                None::<()>
            });
        assert!(
            result.is_err(),
            "a collision must fail before opening the sheet"
        );
        assert!(!opened);
        assert_eq!(std::fs::read(&original).unwrap(), b"existing document");
    }

    #[test]
    fn cancel_and_success_remove_staging_and_allow_the_same_name_again() {
        let documents = Documents::new();
        let exporter = Exporter::new();
        let unrelated = documents.0.join("keep.txt");
        std::fs::write(&unrelated, b"keep").unwrap();
        for outcome in [None, Some("chosen destination")] {
            let path = documents.0.join("report.pdf");
            let result = exporter
                .with_staged_file(&documents.0, "report.pdf", b"export", || {
                    assert_eq!(std::fs::read(&path).unwrap(), b"export");
                    outcome
                })
                .unwrap();
            assert_eq!(result, outcome);
            assert!(!path.exists());
            assert_eq!(std::fs::read(&unrelated).unwrap(), b"keep");
        }
    }

    #[test]
    fn an_overlapping_export_cannot_replace_the_active_file_or_picker() {
        let documents = Documents::new();
        let exporter = Exporter::new();
        let (opened_tx, opened_rx) = std::sync::mpsc::sync_channel(0);
        let (close_tx, close_rx) = std::sync::mpsc::sync_channel(0);
        std::thread::scope(|scope| {
            let exporter_ref = &exporter;
            let document_path = &documents.0;
            let first = scope.spawn(move || {
                exporter_ref.with_staged_file(document_path, "first.pdf", b"first", || {
                    opened_tx.send(()).unwrap();
                    close_rx
                        .recv_timeout(std::time::Duration::from_secs(5))
                        .unwrap();
                })
            });
            opened_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            for name in ["first.pdf", "second.pdf"] {
                let result = exporter.with_staged_file(&documents.0, name, b"second", || {
                    panic!("another sheet must not open during an export");
                });
                assert!(result.is_err());
            }
            assert_eq!(
                std::fs::read(documents.0.join("first.pdf")).unwrap(),
                b"first"
            );
            assert!(!documents.0.join("second.pdf").exists());
            close_tx.send(()).unwrap();
            first.join().unwrap().unwrap();
        });
        assert!(!documents.0.join("first.pdf").exists());
        assert!(exporter
            .with_staged_file(&documents.0, "second.pdf", b"second", || ())
            .is_ok());
    }

    #[test]
    fn a_staging_error_does_not_leave_the_exporter_busy() {
        let documents = Documents::new();
        let exporter = Exporter::new();
        assert!(exporter
            .with_staged_file(&documents.0.join("absent"), "report.pdf", b"export", || {
                panic!("the sheet must not open when staging fails");
            })
            .is_err());
        assert!(exporter
            .with_staged_file(&documents.0, "report.pdf", b"export", || ())
            .is_ok());
        assert!(!documents.0.join("report.pdf").exists());
    }
}
