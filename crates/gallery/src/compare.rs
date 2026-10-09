//! Compare equal-sized upstream and native captures, writing evidence instead of
//! hiding renderer differences behind an arbitrary pass threshold.
use std::{error::Error, path::PathBuf};
use yakui_to_image::image::{self, Rgba, RgbaImage};
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: compare REFERENCE.png NATIVE.png OUTPUT_DIRECTORY".into());
    }
    let reference = image::open(&args[0])?.into_rgba8();
    let native = image::open(&args[1])?.into_rgba8();
    if reference.dimensions() != native.dimensions() {
        return Err("Capture dimensions differ; regenerate at matching viewport and scale".into());
    }
    let (w, h) = reference.dimensions();
    let mut side = RgbaImage::new(w * 2, h);
    let mut diff = RgbaImage::new(w, h);
    let mut squared = 0f64;
    let mut absolute = 0u64;
    let mut changed = 0u64;
    for y in 0..h {
        for x in 0..w {
            let a = reference.get_pixel(x, y);
            let b = native.get_pixel(x, y);
            side.put_pixel(x, y, *a);
            side.put_pixel(x + w, y, *b);
            let mut max = 0u8;
            for channel in 0..3 {
                let d = a[channel].abs_diff(b[channel]);
                max = max.max(d);
                absolute += d as u64;
                squared += (d as f64).powi(2);
            }
            if max > 16 {
                changed += 1;
            }
            diff.put_pixel(x, y, Rgba([max.saturating_mul(4), 0, 0, 255]));
        }
    }
    let out = PathBuf::from(&args[2]);
    std::fs::create_dir_all(&out)?;
    side.save(out.join("side-by-side.png"))?;
    diff.save(out.join("difference.png"))?;
    let metrics = serde_json::json!({"reference":args[0],"native":args[1],"width":w,"height":h,"mean_absolute_rgb_error":absolute as f64/(w as f64*h as f64*3.),"rgb_rmse":(squared/(w as f64*h as f64*3.)).sqrt(),"pixels_differing_by_more_than_16":changed,"changed_fraction":changed as f64/(w as f64*h as f64),"review_required":true});
    std::fs::write(
        out.join("metrics.json"),
        serde_json::to_string_pretty(&metrics)? + "\n",
    )?;
    println!("{metrics}");
    Ok(())
}
