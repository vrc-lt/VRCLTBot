use std::fs;
use std::path::Path;
use std::process::Command;

pub fn convert_pdf_to_png(pdf: &Path) {
    let working_dir = &pdf.parent().unwrap();
    let pdf_cannonical_path = pdf.canonicalize().unwrap();
    let pdf_path_str = pdf_cannonical_path.to_str();
    dbg!("{:?}", &pdf_path_str);
    let _ = Command::new("pdftoppm")
        .current_dir(working_dir)
        .arg("-scale-to-x")
        .arg("1280")
        .arg("-scale-to-y")
        .arg("720")
        .arg("-png")
        .arg(&pdf_path_str.unwrap())
        .arg("image")
        .output()
        .expect("failed to execute process")
        .stdout;
    let exit_status = Command::new("ffmpeg")
        .current_dir(working_dir)
        .arg("-y")
        .arg("-pattern_type")
        .arg("glob")
        .arg("-r")
        .arg("1/2")
        .arg("-i")
        .arg("image-*.png")
        .arg("-c:v")
        .arg("libx264")
        .arg("-r")
        .arg("30")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg("result.mp4")
        .output()
        .expect("failed to execute process")
        .stderr;
    println!("running ffmpeg: {:#?}", String::from_utf8(exit_status));
    println!("conversion finished.");
}
