use crate::console::constants::CARTRIDGE_SIZE;

pub fn read_file(cartridge_path: &str) -> [u8; CARTRIDGE_SIZE] {
    let mut data = [0u8; CARTRIDGE_SIZE];

    #[cfg(not(efi))]
    {
        use std::fs;
        use std::path::Path;
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(cartridge_path);
        data = fs::read(path)
            .expect("Failed to read file")
            .try_into()
            .unwrap();
    }

    #[cfg(efi)]
    {
        use log::info;
        use uefi::boot;
        use uefi::prelude::*;
        use uefi::proto::media::file::*;
        use uefi::proto::media::fs::SimpleFileSystem;

        let fs_handle = boot::get_handle_for_protocol::<SimpleFileSystem>().unwrap();
        let mut fs = boot::open_protocol_exclusive::<SimpleFileSystem>(fs_handle).unwrap();
        let mut root_dir = fs.open_volume().unwrap();

        info!("Opening file: {}", "default.gb");

        let file_handle = root_dir
            .open(cstr16!("\\default.gb"), FileMode::Read, FileAttribute::empty())
            .expect("default.gb not found, make sure it's as esp/");

        let mut file = file_handle
            .into_regular_file()
            .expect("default.gb exists but is not a regular file");

        let mut info_buf = [0u8; 128];
        let info = file.get_info::<FileInfo>(&mut info_buf).unwrap();
        let file_size = info.file_size() as usize;

        if file_size != CARTRIDGE_SIZE {
            info!("Wrong size: {}", file_size);
            boot::stall(5_000_000);
            panic!("Wrong ROM size");
        }

        file.read(&mut data[..file_size]).unwrap();
    }

    data
}