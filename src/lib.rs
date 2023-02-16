use rand::random;
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_dir, remove_file, rename, File, OpenOptions};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

/// A single producer, single consumer filesystem pipe
///
/// This type uses timestamps and atomic filesystem operations
/// (rename) to synchronise a single producer of data with a single
/// consumer of data.
///
/// This crate does not provide any signalling mechanism, which means
/// that pulling must happen in a busy-loop.  Implementing signal
/// handling is left as an excercise to the reader.
pub struct HyperPipe {
    root: PathBuf,
    manifest: Manifest,
    lag_ms: u64,
}

impl HyperPipe {
    /// Create a new instance of Hyperpipe at a particular location
    pub fn new<'p, P: Into<&'p Path>>(path: P, lag_ms: u64) -> Option<Self> {
        let root = path.into().to_path_buf();
        create_dir_all(&root).ok()?;
        Self::initialise(root, lag_ms)
    }

    fn initialise(root: PathBuf, lag_ms: u64) -> Option<Self> {
        let mut manifest = match Manifest::load(&root) {
            Some(m) => m,
            None => {
                eprintln!("Failed to load pipe Manifest!");
                return None;
            }
        };

        // This is a bit of a hack.  For the writing side this doesn't
        // matter because on every write the 'latest' timestamp is
        // updated.  And for the reading side this value is replaced
        // with the UNIX_EPOCH on the first read, meaning that all
        // previously written values will be read.
        //
        // This should probably not be here but eeh.
        manifest.latest = None;
        Some(Self {
            root,
            manifest,
            lag_ms,
        })
    }

    /// Push some data into the pipe
    pub fn push(&mut self, data: Vec<u8>) -> Option<()> {
        // Generate a lower bound on insertion timestamps
        let ts = SystemTime::now();

        // Write the data file
        let data_id = generate_data_id();
        let data_path = self.root.join(&data_id);
        std::thread::sleep(std::time::Duration::from_millis(self.lag_ms));
        let mut data_file = File::create(&data_path).ok()?;
        data_file.write_all(data.as_slice()).ok()?;

        // Then update the manifest
        self.manifest.update(&self.root, ts);
        Some(())
    }

    /// Grab a single entry from the pipe if one exists
    pub fn pull(&mut self) -> Option<Vec<u8>> {
        let latest_manifest = Manifest::load(&self.root)?;

        // eprintln!(
        //     "In Memory: {:#?}, Loaded: {:#?}",
        //     self.manifest, latest_manifest
        // );

        // If the manifest timestamp is None (i.e. we are pulling for
        // the first time in this run) OR if the last timestamp is
        // older than the newly read timestamp (meaning something was
        // inserted)
        if self.manifest.latest.is_none() || self.manifest.latest < latest_manifest.latest {
            // eprintln!(
            //     "{}",
            //     if self.manifest.latest.is_none() {
            //         "in-memory latest is None"
            //     } else {
            //         "bigger (Newer?) timestamp read from manifest"
            //     }
            // );

            // If we are loading for the first time we want to read ALL
            let latest = self.manifest.latest.unwrap_or(SystemTime::UNIX_EPOCH);

            // Iterate over the data directory and find items that are
            // newer than the last pull timestamp
            let mut vec: Vec<_> = read_dir(&self.root)
                .ok()?
                .filter_map(|x| match x {
                    Ok(x) if x.metadata().unwrap().is_file() && x.file_name() != "manifest" => {
                        Some(x)
                    }
                    _ => None,
                })
                .filter(|x| x.metadata().unwrap().created().unwrap() > latest)
                .collect();

            // eprintln!("Found {} data-files!", vec.len());

            // Return the oldest of the entries
            vec.sort_by(|x, y| {
                x.metadata()
                    .unwrap()
                    .created()
                    .unwrap()
                    .partial_cmp(&y.metadata().unwrap().created().unwrap())
                    .unwrap()
            });

            // eprintln!("Sorted by time: {:?}", vec);

            // If no new data has been inserted we can return None
            if vec.is_empty() {
                return None;
            }

            // Otherwise grab the oldest (?) entry from the list and return it
            let entry = vec.remove(0);
            let new_latest = entry.metadata().unwrap().created().unwrap();
            // eprintln!("New latest timestamp: {:?}", new_latest);
            self.manifest.latest = Some(new_latest);

            let mut buf = vec![];
            let mut f = File::open(entry.path()).ok()?;
            f.read_to_end(&mut buf).ok()?;
            // Sleep for duration of lag_ms to ensure that the async is fun
            std::thread::sleep(std::time::Duration::from_millis(self.lag_ms));
            remove_file(entry.path()).ok()?;
            return Some(buf);
        }

        return None;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    latest: Option<SystemTime>,
}

fn manifest_path(root: &PathBuf) -> PathBuf {
    root.join("manifest")
}

fn generate_data_id() -> String {
    let id: usize = random();
    format!("{:x?}.bin", id)
}

/// Write operations are not atomic, meaning that a reader can end up
/// with a partial view of a given file.  To perform an atomic write
/// we first write into a ".swap" file, and then rename it, which is
/// an atomic operation.
fn util_atomic_write(manifest_path: &PathBuf, content: Vec<u8>) -> Option<()> {
    let mut swap_path = manifest_path.clone();
    swap_path.set_extension("swap");

    let mut swap = File::create(&swap_path).unwrap();
    swap.write_all(content.as_slice()).unwrap();
    rename(swap_path, manifest_path).unwrap();
    Some(())
}

impl Manifest {
    fn load(root: &PathBuf) -> Option<Self> {
        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(manifest_path(root))
            .ok()?;

        let mut buf = String::new();

        fn write_default(f: &mut File) -> Option<Manifest> {
            let m = Manifest { latest: None };
            f.write_all(serde_json::to_string(&m).unwrap().as_bytes())
                .unwrap();
            Some(m)
        }

        // Load the manifest and return it.  If an error occurs during
        // reading or parsing instead we generate a new manifest and
        // write it back.
        match f.read_to_string(&mut buf) {
            Ok(_) => serde_json::from_str(&buf)
                .ok()
                .unwrap_or_else(|| write_default(&mut f)),
            Err(_) => write_default(&mut f),
        }
    }

    fn update(&mut self, root: &PathBuf, new_latest: SystemTime) -> Option<()> {
        self.latest = Some(new_latest);
        let buf = serde_json::to_string(self).ok()?;
        util_atomic_write(&manifest_path(root), buf.into_bytes())
    }
}

#[test]
fn ping_pong() {
    let temp_dir = tempfile::tempdir().unwrap();
    let pipe_path = temp_dir.into_path();

    let mut p1 = HyperPipe::new(pipe_path.as_path(), 0).unwrap();
    let v1 = vec![1, 2, 3, 4, 5, 6];
    p1.push(v1.clone()).unwrap();

    let mut p2 = HyperPipe::new(pipe_path.as_path(), 0).unwrap();
    let v2 = p2.pull().unwrap();
    assert_eq!(v1, v2);
}
