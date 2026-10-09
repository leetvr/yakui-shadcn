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
    // Report the union of visible content as well as the whole viewport. Empty
    // margins otherwise make even a badly sized small control score deceptively low.
    let mut bounds = (w, h, 0, 0);
    let background_a = reference.get_pixel(0, 0);
    let background_b = native.get_pixel(0, 0);
    for y in 0..h {
        for x in 0..w {
            if (0..3).any(|channel| {
                reference.get_pixel(x, y)[channel].abs_diff(background_a[channel]) > 8
                    || native.get_pixel(x, y)[channel].abs_diff(background_b[channel]) > 8
            }) {
                bounds.0 = bounds.0.min(x);
                bounds.1 = bounds.1.min(y);
                bounds.2 = bounds.2.max(x + 1);
                bounds.3 = bounds.3.max(y + 1);
            }
        }
    }
    let content = if bounds.2 > bounds.0 && bounds.3 > bounds.1 {
        let mut total = 0f64;
        for y in bounds.1..bounds.3 {
            for x in bounds.0..bounds.2 {
                for channel in 0..3 {
                    total += (reference.get_pixel(x, y)[channel]
                        .abs_diff(native.get_pixel(x, y)[channel])
                        as f64)
                        .powi(2);
                }
            }
        }
        let cw = bounds.2 - bounds.0;
        let ch = bounds.3 - bounds.1;
        let mut cropped = RgbaImage::new(cw * 2, ch);
        for y in 0..ch {
            for x in 0..cw {
                cropped.put_pixel(x, y, *reference.get_pixel(x + bounds.0, y + bounds.1));
                cropped.put_pixel(x + cw, y, *native.get_pixel(x + bounds.0, y + bounds.1));
            }
        }
        Some((cropped, (total / (cw as f64 * ch as f64 * 3.)).sqrt()))
    } else {
        None
    };
    let out = PathBuf::from(&args[2]);
    std::fs::create_dir_all(&out)?;
    side.save(out.join("side-by-side.png"))?;
    diff.save(out.join("difference.png"))?;
    if let Some((cropped, _)) = &content {
        cropped.save(out.join("content-side-by-side.png"))?;
    }
    let metrics = serde_json::json!({"reference":args[0],"native":args[1],"width":w,"height":h,"mean_absolute_rgb_error":absolute as f64/(w as f64*h as f64*3.),"rgb_rmse":(squared/(w as f64*h as f64*3.)).sqrt(),"pixels_differing_by_more_than_16":changed,"changed_fraction":changed as f64/(w as f64*h as f64),"content_bounds":bounds,"content_rgb_rmse":content.map(|(_,rmse)|rmse),"review_required":true});
    std::fs::write(
        out.join("metrics.json"),
        serde_json::to_string_pretty(&metrics)? + "\n",
    )?;
    println!("{metrics}");
    Ok(())
}
