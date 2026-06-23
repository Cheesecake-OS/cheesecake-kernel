use std::path::Path;
use std::process::Command;

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
    println!("Profile: {}", profile);

    if profile == "release" {
        println!("Making vdi file");
        let output_vdi_text = format!(
            "target/x86_64-unknown-none/{}/cheesecake_kernel.vdi",
            profile
        );
        let vdi_path = Path::new(&output_vdi_text);

        if vdi_path.exists() {
            let _ = std::fs::remove_file(vdi_path);
        }
        let status = Command::new("C:\\Program Files\\Oracle\\VirtualBox\\VBoxManage.exe")
            .arg("convertfromraw")
            .arg(output_path) // Ruta del archivo .img de entrada
            .arg(vdi_path) // Ruta del archivo .vdi de salida
            .arg("--format")
            .arg("VDI")
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("VDI image successfully made at: {:?}", vdi_path);
            }
            Ok(s) => {
                eprintln!("VBoxManage exited with an error status: {}", s);
            }
            Err(e) => {
                eprintln!(
                            "Failed to execute VBoxManage. Is VirtualBox installed in the default path? Error: {}",
                            e
                        );
            }
        }
    }
}
