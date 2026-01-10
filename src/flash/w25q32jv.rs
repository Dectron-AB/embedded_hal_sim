use std::sync::{Arc, RwLock};

use embedded_storage::nor_flash::{NorFlash as SyncNorFlash, ReadNorFlash as SyncReadNorFlash};
use embedded_storage_async::nor_flash::{
    NorFlash as AsyncNorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash as AsyncReadNorFlash,
};

pub const PAGE_SIZE: u32 = 256;
pub const SECTOR_SIZE: u32 = PAGE_SIZE * 16;

pub struct W25q32jv {
    data: Arc<RwLock<Box<[u8]>>>,
}

impl W25q32jv {
    pub fn new(data: Arc<RwLock<Box<[u8]>>>) -> Self {
        Self { data }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    //SpiError(S),
    //PinError(P),
    NotAligned,
    OutOfBounds,
    WriteEnableFail,
    ReadbackFail,
}

impl NorFlashError for Error {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Error::NotAligned => NorFlashErrorKind::NotAligned,
            Error::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            _ => NorFlashErrorKind::Other,
        }
    }
}

impl embedded_storage_async::nor_flash::ErrorType for W25q32jv {
    type Error = Error;
}

impl SyncReadNorFlash for W25q32jv {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        let data = self.data.read().unwrap();
        bytes.copy_from_slice(&data[offset..(offset + bytes.len())]);

        Ok(())
    }

    fn capacity(&self) -> usize {
        self.data.read().unwrap().len()
    }
}

impl AsyncReadNorFlash for W25q32jv {
    const READ_SIZE: usize = 1;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        SyncReadNorFlash::read(self, offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.data.read().unwrap().len()
    }
}

impl SyncNorFlash for W25q32jv {
    const WRITE_SIZE: usize = 1;

    const ERASE_SIZE: usize = SECTOR_SIZE as usize;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if !from.is_multiple_of(SECTOR_SIZE) {
            return Err(Error::NotAligned);
        }

        if !to.is_multiple_of(SECTOR_SIZE) {
            return Err(Error::NotAligned);
        }

        if from > to {
            return Err(Error::OutOfBounds);
        }

        let mut data = self.data.write().unwrap();
        data[from as usize..to as usize]
            .iter_mut()
            .for_each(|b| *b = 0xFF);

        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let offset = offset as usize;
        let mut data = self.data.write().unwrap();

        for (dst, src) in data[offset..].iter_mut().zip(bytes) {
            *dst &= src;
        }

        Ok(())
    }
}

impl AsyncNorFlash for W25q32jv {
    const WRITE_SIZE: usize = 1;

    const ERASE_SIZE: usize = SECTOR_SIZE as usize;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        SyncNorFlash::erase(self, from, to)
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        SyncNorFlash::write(self, offset, bytes)
    }
}

#[cfg(test)]
mod test {
    use super::Error;
    use crate::flash::w25q32jv::{SECTOR_SIZE, W25q32jv};
    use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
    use std::sync::{Arc, RwLock};

    #[test]
    fn test() {
        let mut data = vec![0xFFu8; SECTOR_SIZE as usize].into_boxed_slice();
        data[0] = 0;
        data[1] = 1;
        data[2] = 2;
        data[3] = 3;

        let data = RwLock::new(data);
        let data = Arc::new(data);

        let mut flash = W25q32jv::new(Arc::clone(&data));
        let mut dst = [0; 5];
        flash.read(0, &mut dst).unwrap();
        assert_eq!(dst, [0, 1, 2, 3, 0xFF]);

        // Write ones (can only change a bit from 1 to zero without erase) so writing 0xFF should change nothing
        flash.write(0, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]).unwrap();
        flash.read(0, &mut dst).unwrap();
        assert_eq!(dst, [0, 1, 2, 3, 0xFF]);

        // Write zeros (can only change a bit from 1 to zero without erase) so writing 0x00 clear all bits
        flash.write(0, &[0x00, 0x00, 0x00, 0x00, 0x00]).unwrap();
        flash.read(0, &mut dst).unwrap();
        assert_eq!(dst, [0, 0, 0, 0, 0]);

        // Erasing should set all bits to 1
        flash.erase(0, SECTOR_SIZE).unwrap();
        flash.read(0, &mut dst).unwrap();
        assert_eq!(dst, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);

        {
            // Bulk write, single reads
            let mut read_bytes = [0; 1];
            let bytes_to_write = [0x12, 0x34, 0x56, 0x78, 0x9a];
            flash.write(0, &bytes_to_write).unwrap();
            for (i, expected_byte) in bytes_to_write.iter().enumerate() {
                flash.read(i as u32, &mut read_bytes).unwrap();
                assert_eq!(read_bytes[0], *expected_byte);
            }
        }

        flash.erase(0, SECTOR_SIZE).unwrap();
        assert_eq!(flash.erase(0, 2), Err(Error::NotAligned));
        assert_eq!(flash.erase(1, SECTOR_SIZE), Err(Error::NotAligned));
        assert_eq!(flash.erase(SECTOR_SIZE, 0), Err(Error::OutOfBounds));
        //assert_eq!(flash.erase(0, 2 * SECTOR_SIZE), Err(Error::OutOfBounds));

        {
            // single reads write, bulk read
            let mut read_bytes = [0; 5];
            let bytes_to_write = [0x12, 0x34, 0x56, 0x78, 0x9a];

            for (i, byte_to_write) in bytes_to_write.iter().enumerate() {
                flash.write(i as u32, &[*byte_to_write]).unwrap();
            }

            flash.read(0, &mut read_bytes).unwrap();
            assert_eq!(read_bytes, bytes_to_write);
        }
    }
}
