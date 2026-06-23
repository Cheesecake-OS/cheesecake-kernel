use std::path::Path;

fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());

    let output_kernel_path = &format!("target/x86_64-unknown-none/{}/cheesecake_kernel", profile);
    let kernel_path = Path::new(output_kernel_path);
    let output_formated_text = &format!(
        "target/x86_64-unknown-none/{}/cheesecake_kernel.img",
        profile
    );
    let output_path = Path::new(output_formated_text);
    let mut boot = bootloader::UefiBoot::new(kernel_path);
    boot.create_disk_image(output_path).unwrap();
    println!("Disk image made at: {:?}", output_path);
    println!("Profile: {}", profile)
}
