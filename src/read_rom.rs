use crate::console::constants::CARTRIDGE_SIZE;

pub fn read_file(cartridge_path: &str) -> [u8; CARTRIDGE_SIZE] {
    let mut data = [0u8; CARTRIDGE_SIZE];

    #[cfg(not(efi))]
    {
        use std::fs;
        use std::path::Path;
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(cartridge_path);
        let rom_data = fs::read(path).expect("Failed to read file");
        let len = rom_data.len().min(CARTRIDGE_SIZE);
        data[..len].copy_from_slice(&rom_data[..len]);
    }

    #[cfg(efi)]
    {
        use log::info;
        use uefi::boot;
        use uefi::prelude::*;
        use uefi::proto::loaded_image::LoadedImage;
        use uefi::proto::media::file::*;
        use uefi::proto::media::fs::SimpleFileSystem;

        list_efi_root();

        let loaded_image =
            boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle()).unwrap();
        let device_handle = loaded_image.device().unwrap();
        let mut fs = boot::open_protocol_exclusive::<SimpleFileSystem>(device_handle).unwrap();
        let mut root_dir = fs.open_volume().unwrap();

        info!("oppening file: {}", cartridge_path);

        let file_handle = root_dir
            .open(
                cstr16!("default.gb"),
                FileMode::Read,
                FileAttribute::empty(),
            )
            .expect("default.gb not found, make sure it's in the USB root");

        let mut file = file_handle
            .into_regular_file()
            .expect("default.gb exists but is not a regular file");

        let mut info_buf = [0u8; 128];
        let info = file.get_info::<FileInfo>(&mut info_buf).unwrap();
        let file_size = info.file_size() as usize;

        if file_size > CARTRIDGE_SIZE {
            info!("File too large: {}", file_size);
            boot::stall(5_000_000_000);
            panic!("ROM size exceeds CARTRIDGE_SIZE");
        }

        file.read(&mut data[..file_size]).unwrap();
    }

    data
}

#[cfg(efi)]
pub fn list_efi_root() {
    use log::info;
    use uefi::boot;
    use uefi::proto::loaded_image::LoadedImage;
    use uefi::proto::media::file::*;
    use uefi::proto::media::fs::SimpleFileSystem;

    let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle()).unwrap();
    let device_handle = loaded_image.device().unwrap();

    let mut fs = boot::open_protocol_exclusive::<SimpleFileSystem>(device_handle).unwrap();
    let mut root_dir = fs.open_volume().unwrap();

    let mut buf = [0u8; 512];
    loop {
        match root_dir.read_entry(&mut buf) {
            Ok(Some(entry)) => info!("  {} size={}", &entry.file_name(), entry.file_size()),
            Ok(None) => break,
            Err(e) => {
                info!("error: {:?}", e);
                break;
            }
        }
    }
}
