use std::path::Path;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt as UnixOpenOptionsExt;

pub struct SharedFile;

impl SharedFile {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<std::fs::File> {
        OpenOptions::new()
            .read(true)
            .open_shared(path)
    }
}

pub struct OwnedFile;

impl OwnedFile {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<std::fs::File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open_exclusive(path)
    }

    pub fn create<P: AsRef<Path>>(path: P) -> std::io::Result<std::fs::File> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .open_exclusive(path)
    }
}

pub struct OpenOptions {
    sys: std::fs::OpenOptions,
    #[cfg(unix)]
    flags: i32,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self {
            sys: std::fs::OpenOptions::new(),
            #[cfg(unix)]
            flags: 0,
        }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.sys.read(read);

        self
    }

    pub fn write(&mut self, write: bool) -> &mut Self {
        self.sys.write(write);

        self
    }

    pub fn append(&mut self, append: bool) -> &mut Self {
        self.sys.append(append);

        self
    }

    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.sys.truncate(truncate);

        self
    }

    pub fn create(&mut self, create: bool) -> &mut Self {
        self.sys.create(create);

        self
    }

    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.sys.create_new(create_new);

        self
    }

    pub fn open_exclusive<P: AsRef<Path>>(&self, path: P) -> std::io::Result<std::fs::File> {
        self.sys.clone()
            .custom_flags(self.flags | libc::O_EXLOCK)
            .open(path)
    }

    pub fn open_shared<P: AsRef<Path>>(&self, path: P) -> std::io::Result<std::fs::File> {
        self.sys.clone()
            .custom_flags(self.flags | libc::O_SHLOCK)
            .open(path)
    }
}

#[cfg(unix)]
impl UnixOpenOptionsExt for OpenOptions {
    fn mode(&mut self, mode: u32) -> &mut Self {
        self.sys.mode(mode);

        self
    }

    fn custom_flags(&mut self, flags: i32) -> &mut Self {
        self.flags = flags;

        self
    }
}
