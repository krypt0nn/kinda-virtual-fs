use std::collections::HashMap;
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::env::temp_dir;

#[derive(Debug, Clone)]
pub struct Entry {
    pub bytes: Vec<u8>,
    random_id: String
}

impl Entry {
    /// Create new entry
    pub fn new<T: Into<Vec<u8>>>(bytes: T) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));

        let bytes = bytes.into().to_vec();
        let random_id = format!("{:x}-{:x}.kvfs", timestamp.as_micros(), bytes.len());

        Self {
            bytes,
            random_id
        }
    }

    fn get_temp_path(&self) -> String {
        format!("{}/{}", temp_dir().to_string_lossy(), self.random_id)
    }

    /// Map entry to physical location in your filesystem
    /// 
    /// Method returns path to the mapped file, or filesystem writing error
    /// 
    /// ```
    /// use kinda_virtual_fs::*;
    /// 
    /// let entry = Entry::new("Hello, World!");
    /// 
    /// let file_path = entry.map().expect("Failed to map entry");
    /// let file_content = std::fs::read_to_string(file_path).expect("Failed to read mapped entry");
    /// 
    /// assert_eq!(&file_content, "Hello, World!");
    /// ```
    pub fn map(&self) -> std::io::Result<String> {
        let path = self.get_temp_path();

        if !Path::new(&path).exists() {
            std::fs::write(&path, self.bytes.as_slice())?;
        }

        Ok(path)
    }

    /// Unmap (delete) entry from your filesystem
    /// 
    /// Entry will be automatically unmapped when its value is no more needed
    /// 
    /// ## Manual unmapping
    /// 
    /// ```
    /// use kinda_virtual_fs::*;
    /// 
    /// let entry = Entry::new("Hello, World!");
    /// 
    /// let path = entry.map().unwrap();
    /// 
    /// assert_eq!(std::path::Path::new(&path).exists(), true);
    /// 
    /// entry.unmap();
    /// 
    /// assert_eq!(std::path::Path::new(&path).exists(), false);
    /// ```
    /// 
    /// ## Automatic unmapping
    /// 
    /// ```
    /// use kinda_virtual_fs::*;
    /// 
    /// let path = {
    ///     let entry = Entry::new("Hello, World!");
    ///     let path = entry.map().unwrap();
    /// 
    ///     assert_eq!(std::path::Path::new(&path).exists(), true);
    /// 
    ///     path
    /// };
    /// 
    /// // entry is dropped here because it's no more used
    /// 
    /// assert_eq!(std::path::Path::new(&path).exists(), false);
    /// ```
    pub fn unmap(&self) -> std::io::Result<()> {
        let path = self.get_temp_path();

        if Path::new(&path).exists() {
            std::fs::remove_file(path)
        }

        else {
            Ok(())
        }
    }

    /// Get bytes stored in entry
    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl<T> From<T> for Entry where T: Into<Vec<u8>> {
    fn from(bytes: T) -> Self {
        Self::new(bytes)
    }
}

impl Drop for Entry {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.unmap();
    }
}

#[derive(Debug, Clone, Default)]
pub struct Storage {
    pub entries: HashMap<String, Entry>
}

impl Storage {
    /// Create storage
    pub fn new(entries: HashMap<String, Entry>) -> Self {
        Self { entries }
    }

    /// Add new entry to storage
    /// 
    /// Works as `HashMap::insert` method, so will return `Some(Entry)` if it replaced older value
    /// 
    /// ```
    /// use kinda_virtual_fs::*;
    /// 
    /// let mut storage = Storage::default();
    /// 
    /// storage.add("example 1", "Hello, World!");
    /// storage.add("example 2", Entry::new("Also accepts Entry struct"));
    /// ```
    pub fn add<K: ToString, T: Into<Entry>>(&mut self, key: K, entry: T) -> Option<Entry> {
        self.entries.insert(key.to_string(), entry.into())
    }

    /// Get entry with the given key
    pub fn get<T: ToString>(&self, key: T) -> Option<&Entry> {
        self.entries.get(&key.to_string())
    }

    /// Remove entry with the given key
    pub fn remove<T: ToString>(&mut self, key: T) -> Option<Entry> {
        self.entries.remove(&key.to_string())
    }

    /// Try to map entry with specific key
    /// 
    /// ```
    /// use kinda_virtual_fs::*;
    /// 
    /// let mut storage = Storage::default();
    /// 
    /// storage.add("example", "Hello, World!");
    /// 
    /// let file_path = storage.map("example").expect("Failed to map entry");
    /// let file_content = std::fs::read_to_string(file_path).expect("Failed to read mapped entry");
    /// 
    /// assert_eq!(&file_content, "Hello, World!");
    /// ```
    pub fn map<T: ToString>(&self, key: T) -> std::io::Result<String> {
        match self.get(key.to_string()) {
            Some(entry) => entry.map(),
            None => Err(Error::new(ErrorKind::Other, format!("No entry with key {} found", key.to_string())))
        }
    }

    /// Unmap entry with specific key
    /// 
    /// Will return `Ok(())` if there's no entry with specified key
    /// 
    /// Entry will be automatically unmapped when its value is no more needed
    /// 
    /// ## Manual unmapping
    /// 
    /// ```
    /// use kinda_virtual_fs::*;
    /// 
    /// let mut storage = Storage::default();
    /// 
    /// storage.add("example", "Hello, World!");
    /// 
    /// let path = storage.map("example").unwrap();
    /// 
    /// assert_eq!(std::path::Path::new(&path).exists(), true);
    /// 
    /// storage.unmap("example");
    /// 
    /// assert_eq!(std::path::Path::new(&path).exists(), false);
    /// ```
    /// 
    /// ## Automatic unmapping
    /// 
    /// ```
    /// use kinda_virtual_fs::*;
    /// 
    /// let mut storage = Storage::default();
    /// 
    /// storage.add("example", "Hello, World!");
    /// 
    /// let path = storage.map("example").unwrap();
    /// 
    /// assert_eq!(std::path::Path::new(&path).exists(), true);
    /// 
    /// storage.remove("example");
    /// 
    /// assert_eq!(std::path::Path::new(&path).exists(), false);
    /// ```
    pub fn unmap<T: ToString>(&self, key: T) -> std::io::Result<()> {
        match self.get(key.to_string()) {
            Some(entry) => entry.unmap(),
            None => Ok(())
        }
    }
}
