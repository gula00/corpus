use bootloader::DiskImageBuilder;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let kernel_path = env::args().nth(1).expect("Usage: os-boot <kernel-binary>");
    let kernel_path = PathBuf::from(kernel_path);

    let out_dir = kernel_path.parent().unwrap();
    let bios_image = out_dir.join("blog_os-bios.img");
    let uefi_image = out_dir.join("blog_os-uefi.img");

    let builder = DiskImageBuilder::new(kernel_path);
    builder.create_bios_image(&bios_image).unwrap();
    println!("Created BIOS image: {}", bios_image.display());

    builder.create_uefi_image(&uefi_image).unwrap();
    println!("Created UEFI image: {}", uefi_image.display());
}
