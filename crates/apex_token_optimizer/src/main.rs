use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
struct Coord { x: f64, y: f64 }

fn correct_coord(out: Coord, screen_w: f64, screen_h: f64, img_w: f64, img_h: f64) -> Coord {
    Coord {
        x: out.x * (screen_w / img_w),
        y: out.y * (screen_h / img_h),
    }
}

fn token_reserve(text_tokens: usize, image_tokens: &[usize], keep_latest: usize) -> usize {
    let start = image_tokens.len().saturating_sub(keep_latest);
    text_tokens + image_tokens[start..].iter().sum::<usize>()
}

fn effort_valid(total: f64, waste: f64) -> f64 {
    (total - waste).max(0.0)
}

fn effort_efficiency(total: f64, waste: f64) -> f64 {
    if total <= 0.0 { 0.0 } else { effort_valid(total, waste) / total }
}

fn list_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() { files.push(p); }
        }
    }
    files
}

fn purify_dir(dir: &Path, keep_latest: usize) -> std::io::Result<usize> {
    let mut files = list_files(dir);
    files.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH));
    let remove_count = files.len().saturating_sub(keep_latest);
    let mut removed = 0;
    for p in files.into_iter().take(remove_count) {
        fs::remove_file(&p)?;
        removed += 1;
    }
    Ok(removed)
}

fn arg_value(args: &[String], key: &str, default: &str) -> String {
    args.windows(2)
        .find(|w| w[0] == key)
        .map(|w| w[1].clone())
        .unwrap_or_else(|| default.to_string())
}

fn usage() {
    eprintln!("usage:");
    eprintln!("  apex_token_optimizer correct --x 100 --y 50 --sw 1920 --sh 1080 --iw 1000 --ih 500");
    eprintln!("  apex_token_optimizer reserve --text 120 --imgs 1000,1200,900,800 --keep 3");
    eprintln!("  apex_token_optimizer effort --total 100 --waste 30");
    eprintln!("  apex_token_optimizer purify --dir /path/screens --keep 3");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { usage(); return; }
    match args[1].as_str() {
        "correct" => {
            let x: f64 = arg_value(&args, "--x", "0").parse().unwrap_or(0.0);
            let y: f64 = arg_value(&args, "--y", "0").parse().unwrap_or(0.0);
            let sw: f64 = arg_value(&args, "--sw", "1920").parse().unwrap_or(1920.0);
            let sh: f64 = arg_value(&args, "--sh", "1080").parse().unwrap_or(1080.0);
            let iw: f64 = arg_value(&args, "--iw", "1920").parse().unwrap_or(1920.0);
            let ih: f64 = arg_value(&args, "--ih", "1080").parse().unwrap_or(1080.0);
            let c = correct_coord(Coord { x, y }, sw, sh, iw, ih);
            println!("{{\"x\":{:.3},\"y\":{:.3},\"formula\":\"X_real=X_out*(W_screen/W_img),Y_real=Y_out*(H_screen/H_img)\"}}", c.x, c.y);
        }
        "reserve" => {
            let text: usize = arg_value(&args, "--text", "0").parse().unwrap_or(0);
            let keep: usize = arg_value(&args, "--keep", "3").parse().unwrap_or(3);
            let imgs_raw = arg_value(&args, "--imgs", "");
            let imgs: Vec<usize> = imgs_raw.split(',').filter_map(|s| s.trim().parse().ok()).collect();
            let total = token_reserve(text, &imgs, keep);
            let dropped = imgs.len().saturating_sub(keep);
            println!("{{\"token_reserve\":{},\"kept_frames\":{},\"dropped_frames\":{}}}", total, imgs.len().min(keep), dropped);
        }
        "effort" => {
            let total: f64 = arg_value(&args, "--total", "0").parse().unwrap_or(0.0);
            let waste: f64 = arg_value(&args, "--waste", "0").parse().unwrap_or(0.0);
            println!("{{\"effort_valid\":{:.3},\"efficiency\":{:.6}}}", effort_valid(total, waste), effort_efficiency(total, waste));
        }
        "purify" => {
            let dir = PathBuf::from(arg_value(&args, "--dir", "."));
            let keep: usize = arg_value(&args, "--keep", "3").parse().unwrap_or(3);
            match purify_dir(&dir, keep) {
                Ok(n) => println!("{{\"removed\":{},\"kept_latest\":{},\"ts\":{}}}", n, keep, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
                Err(e) => { eprintln!("purify_error={}", e); std::process::exit(1); }
            }
        }
        _ => usage(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coordinate_scales() {
        let c = correct_coord(Coord{x:100.0,y:50.0}, 1920.0, 1080.0, 1000.0, 500.0);
        assert!((c.x - 192.0).abs() < 1e-9);
        assert!((c.y - 108.0).abs() < 1e-9);
    }
    #[test]
    fn reserve_keeps_latest_three() {
        assert_eq!(token_reserve(100, &[1000,1200,900,800], 3), 3000);
    }
    #[test]
    fn effort_clamps() {
        assert_eq!(effort_valid(10.0, 20.0), 0.0);
        assert!((effort_efficiency(100.0, 25.0) - 0.75).abs() < 1e-9);
    }
}
