use std::path::Path;

fn main() {
    let kernel_path = Path::new("target/x86_64-unknown-none/debug/cheesecake_kernel");
    let output_path = Path::new("target/x86_64-unknown-none/debug/cheesecake_kernel.img");
    let mut boot = bootloader::UefiBoot::new(kernel_path);
    boot.create_disk_image(output_path).unwrap();
    println!("Disk image made at: {:?}", output_path);
}
