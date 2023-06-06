use std::ffi::CString;

use std::fs::{OpenOptions,File};
use std::path::PathBuf;


pub struct Fifo{
    path: PathBuf
}

impl Fifo{
    pub fn new(path: PathBuf) -> Fifo{
        Fifo{
            path
        }
    }

    pub fn exists(&self) -> bool{
        self.path.exists()
    }

    pub fn create(&self){
        let filename = CString::new(self.path.as_os_str().to_str().unwrap()).expect("CString could not be created from Fifo's path");
        unsafe {
            libc::mkfifo(filename.as_ptr(), 0o644);
        }
    }

    pub fn open_write(&self) -> File{
        OpenOptions::new()
        .read(false)
        .write(true)
        .open(&self.path).expect("File could not be opened in write mode")
    }

    pub fn open_read(&self) -> File{
        OpenOptions::new()
        .read(true)
        .write(false)
        .open(&self.path).expect("File could not be opened in read mode")
    }
}