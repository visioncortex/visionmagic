use super::painter::*;

pub struct Smoother {
    pub im: Image,
    pub blurriness: u32,
}

macro_rules! rgb_acc {
    ($r: expr, $g: expr, $b: expr, $w: expr, $im: expr, $xx: expr, $yy: expr) => {
        $r += $w * (*elem_v!($im, $xx, $yy, 0) as u32);
        $g += $w * (*elem_v!($im, $xx, $yy, 1) as u32);
        $b += $w * (*elem_v!($im, $xx, $yy, 2) as u32);
    };
}

macro_rules! inside {
    ($mask: expr, $w: expr, $x: expr, $y: expr) => {
        $mask[($w as usize * $y as usize + $x as usize) * 4 + 3] == 255
    };
}

impl Smoother {
    pub fn new(buf: Vec<u8>, width: u32, height: u32, blurriness: u32) -> Self {
        let im = Image { buf, width, height };
        Self { im, blurriness }
    }

    pub fn smooth(self, mask: &[u8]) -> Self {
        let mut im = self.im;
        let radius = self.blurriness;
        assert!(im.width > 2 * radius);
        assert!(im.height > 2 * radius);
        let mut overlay = Vec::new();
        for y in radius..im.height - radius {
            for x in radius..im.width - radius {
                if inside!(mask, im.width, x, y) {
                    // overlay.push((x, y, Self::radial_blur(&im, x, y, radius)));
                    // overlay.push((x, y, Self::radial_blur_edge(&im, mask, x, y, radius)));
                    // overlay.push((x, y, Self::radial_blur_edge_peel(&im, mask, x, y, radius)));
                    overlay.push((x, y, Self::radial_blur_edge_peel_var(&im, mask, x, y, radius)));
                }
            }
        }
        for o in overlay.iter() {
            *elem!(im, o.0, o.1, 0) = ((o.2 >> 24) & 0xFF) as u8;
            *elem!(im, o.0, o.1, 1) = ((o.2 >> 16) & 0xFF) as u8;
            *elem!(im, o.0, o.1, 2) = ((o.2 >> 8) & 0xFF) as u8;
        }
        if true {
            // must apply denoise if using radial_blur_edge_peel_var
            let mut denoise = Vec::new();
            for o in overlay.iter() {
                let x = o.0;
                let y = o.1;
                denoise.push((x, y, Self::denoise(&im, x, y)));
            }
            for o in denoise.iter() {
                *elem!(im, o.0, o.1, 0) = ((o.2 >> 24) & 0xFF) as u8;
                *elem!(im, o.0, o.1, 1) = ((o.2 >> 16) & 0xFF) as u8;
                *elem!(im, o.0, o.1, 2) = ((o.2 >> 8) & 0xFF) as u8;
            }
        }
        Self { im, blurriness: radius }
    }

    #[allow(clippy::many_single_char_names)]
    pub fn radial_blur(im: &Image, x: u32, y: u32, radius: u32) -> u32 {
        let x = x as i32;
        let y = y as i32;
        let radius = radius as i32;
        let radius_square = radius * radius;
        let (mut c, mut r, mut g, mut b) = (0, 0, 0, 0);
        for j in -radius..radius {
            for i in -radius..radius {
                if (i * i + j * j) < radius_square {
                    c += 1;
                    rgb_acc!(r, g, b, 1, im, x + i, y + j);
                }
            }
        }
        ((r / c) << 24) | ((g / c) << 16) | ((b / c) << 8)
    }

    #[allow(clippy::many_single_char_names)]
    pub fn radial_blur_edge(im: &Image, mask: &[u8], x: u32, y: u32, radius: u32) -> u32 {
        let x = x as i32;
        let y = y as i32;
        let radius = radius as i32;
        let radius_square = radius * radius;
        let (mut c, mut r, mut g, mut b) = (0, 0, 0, 0);
        for j in -radius..radius {
            for i in -radius..radius {
                let dd = i * i + j * j;
                if dd < radius_square {
                    let xx = x + i;
                    let yy = y + j;
                    let w = if inside!(mask, im.width, xx, yy) {
                        1
                    } else if dd < 9 {
                        // known region has more influence at the edge
                        128
                    } else {
                        1
                    };
                    c += w;
                    rgb_acc!(r, g, b, w, im, xx, yy);
                }
            }
        }
        ((r / c) << 24) | ((g / c) << 16) | ((b / c) << 8)
    }

    #[allow(clippy::many_single_char_names)]
    pub fn radial_blur_edge_peel(im: &Image, mask: &[u8], x: u32, y: u32, radius: u32) -> u32 {
        let x = x as i32;
        let y = y as i32;
        let radius = radius as i32;
        let (mut c, mut r, mut g, mut b) = (0, 0, 0, 0);
        for ring in 1..radius {
            let ring_sq = ring * ring;
            for j in -ring..ring + 1 {
                for i in -ring..ring + 1 {
                    let dd = i * i + j * j;
                    if ring_sq - 1 <= dd && dd <= ring_sq {
                        let xx = x + i;
                        let yy = y + j;
                        let w = if inside!(mask, im.width, xx, yy) {
                            1
                        } else if dd < 9 {
                            // known region has more influence at the edge
                            64
                        } else {
                            1
                        };
                        c += w;
                        rgb_acc!(r, g, b, w, im, xx, yy);
                    }
                }
            }
        }
        ((r / c) << 24) | ((g / c) << 16) | ((b / c) << 8)
    }

    #[allow(clippy::many_single_char_names)]
    pub fn radial_blur_edge_peel_var(
        im: &Image,
        mask: &[u8],
        x: u32,
        y: u32,
        radius: u32,
    ) -> u32 {
        let x = x as i32;
        let y = y as i32;
        let radius = radius as i32;
        let (mut c, mut r, mut g, mut b) = (0, 0, 0, 0);
        let (mut sum, mut sqsum) = (0, 0);
        for ring in 1..radius {
            let ring_sq = ring * ring;
            for j in -ring..ring + 1 {
                for i in -ring..ring + 1 {
                    let dd = i * i + j * j;
                    if ring_sq - 1 <= dd && dd <= ring_sq {
                        let xx = x + i;
                        let yy = y + j;
                        let w = if inside!(mask, im.width, xx, yy) {
                            1
                        } else if dd < 9 {
                            // known region has more influence at the edge
                            64
                        } else {
                            1
                        };
                        c += w;
                        rgb_acc!(r, g, b, w, im, xx, yy);
                        let s = r + g + b;
                        sum += s;
                        sqsum += s * s;
                    }
                }
            }
            let variance = (12800 * sqsum - 12800 * sum * sum / c) / (12800 * c - 12800);
            if variance > 12800 * radius as u32 / 2 {
                break;
            }
        }
        ((r / c) << 24) | ((g / c) << 16) | ((b / c) << 8)
    }
    
    #[allow(clippy::many_single_char_names)]
    pub fn denoise(im: &Image, x: u32, y: u32) -> u32 {
        let c = 4;
        let (mut r, mut g, mut b) = (0, 0, 0);
        rgb_acc!(r, g, b, 1, im, x - 1, y);
        rgb_acc!(r, g, b, 1, im, x + 1, y);
        rgb_acc!(r, g, b, 1, im, x, y - 1);
        rgb_acc!(r, g, b, 1, im, x, y + 1);
        ((r / c) << 24) | ((g / c) << 16) | ((b / c) << 8)
    }
}
