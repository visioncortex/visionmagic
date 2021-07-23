use super::bitmask;
use super::min_float::MinFloat;
use std::collections::BinaryHeap;
#[derive(Default)]
pub struct Image {
    pub buf: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    pub priority: MinFloat,
    pub i: u32,
}

#[derive(Default)]
pub struct Painter {
    pub im: Image,
    pub inside: bitmask::Bitmask,
    pub times: Vec<f32>,
    pub queue: BinaryHeap<Node>,
    pub count: usize,
    pub max_queue: u32,
    pub progress: u32,
}

impl Painter {
    pub fn new(buf: Vec<u8>, mask: &[u8], width: u32, height: u32) -> Self {
        let len: usize = (width * height) as usize;
        let im = Image { buf, width, height };
        let mut inside = bitmask::new(len);
        let mut times = vec![0.0; len];
        let mut queue = BinaryHeap::with_capacity(len);
        for i in (0..mask.len()).step_by(4) {
            if mask[i + 3] == 255 {
                bitmask::set(&mut inside, i / 4);
                times[i / 4] = 1e6;
            }
        }

        for y in 0..height {
            for x in 0..width {
                let i = index!(im, x, y);
                if !bitmask::get(&inside, i)
                    && ((x > 0 && bitmask::get(&inside, index!(im, x - 1, y)))
                        || (y > 0 && bitmask::get(&inside, index!(im, x, y - 1)))
                        || (x + 1 < width && bitmask::get(&inside, index!(im, x + 1, y)))
                        || (y + 1 < height && bitmask::get(&inside, index!(im, x, y + 1))))
                {
                    queue.push(Node {
                        priority: MinFloat(0.0),
                        i: i as u32,
                    });
                }
            }
        }

        Self {
            im,
            inside,
            times,
            queue,
            count: 0,
            max_queue: 0,
            progress: 0,
        }
    }

    pub fn paint(self) -> Self {
        paint(self.im, self.inside, self.times, self.queue, self.count, self.max_queue, self.progress)
    }
}

macro_rules! paint_detail {
    ($im:expr, $inside:expr, $times:expr, $queue:expr, $x:expr, $y:expr) => {{
        let n = index!($im, $x, $y);
        bitmask::unset(&mut $inside, n);
        inpaint(&mut $im, &$inside, &$times, n);
        $times[n] = min4!(
            solve(
                &$inside,
                &$times,
                index!($im, $x - 1, $y),
                index!($im, $x, $y - 1)
            ),
            solve(
                &$inside,
                &$times,
                index!($im, $x + 1, $y),
                index!($im, $x, $y - 1)
            ),
            solve(
                &$inside,
                &$times,
                index!($im, $x - 1, $y),
                index!($im, $x, $y + 1)
            ),
            solve(
                &$inside,
                &$times,
                index!($im, $x + 1, $y),
                index!($im, $x, $y + 1)
            )
        );
        $queue.push(Node {
            priority: MinFloat($times[n]),
            i: n as u32,
        });
    }};
}

pub fn paint(
    mut im: Image,
    mut inside: bitmask::Bitmask,
    mut times: Vec<f32>,
    mut queue: BinaryHeap<Node>,
    mut count: usize,
    mut max_queue: u32,
    mut progress: u32,
) -> Painter {
    if count == 0 && max_queue == 0 {
        max_queue = queue.len() as u32;
    }
    let max_count = count + 500;
    let progress_start = progress;
    loop {
        match queue.pop() {
            Some(Node { i, .. }) => {
                if count > max_count {
                    if ((max_queue as f32) - (queue.len() as f32)) < ((max_queue as f32) * 0.03) {
                        progress = ((((max_queue as f32) - (queue.len() as f32)) / ((max_queue as f32) * 0.03)) * 97.0) as u32;
                    } else {
                        progress = ((((max_queue as f32) - (queue.len() as f32)) / ((max_queue as f32) * 0.95)) * 0.03 + 97.0) as u32;
                    }
                    if progress < progress_start {
                        progress = progress_start;
                    }
                    break
                }
                let x = i % im.width;
                let y = i / im.width;
                if x > 0 && bitmask::get(&inside, (i - 1) as usize) {
                    paint_detail!(im, inside, times, queue, x - 1, y);
                }
                if x + 1 < im.width && bitmask::get(&inside, (i + 1) as usize) {
                    paint_detail!(im, inside, times, queue, x + 1, y);
                }
                if y > 0 && bitmask::get(&inside, (i - im.width) as usize) {
                    paint_detail!(im, inside, times, queue, x, y - 1);
                }
                if y + 1 < im.height && bitmask::get(&inside, (i + im.width) as usize) {
                    paint_detail!(im, inside, times, queue, x, y + 1);
                }
                count += 1;
            },
            None => {
                progress = 100;
                break;
            },
        }
    }
    Painter {
        im,
        inside,
        times,
        queue,
        count,
        max_queue,
        progress,
    }
}

#[inline]
#[allow(clippy::many_single_char_names)]
fn inpaint(im: &mut Image, inside: &[u32], times: &[f32], p: usize) {
    let point = (p as i32 % im.width as i32, p as i32 / im.width as i32);
    let grad_t = grad_t(im, inside, times, p);
    let radius = 3;
    let r_sq = radius * radius;
    let left = point.0.saturating_sub(radius);
    let right = std::cmp::max(point.0 + radius, im.width as i32 - 1);
    let up = point.1.saturating_sub(radius);
    let down = std::cmp::max(point.1 + radius, im.height as i32 - 1);
    let mut sum = [0.0; 3];
    let mut total = 0.0;
    let mut y = up as i32;

    while y < down {
        let mut x = left as i32;
        while x < right {
            let dx = x - point.0 as i32;
            let dy = y - point.1 as i32;
            if (x != point.0 || y != point.1)
                && dx * dx + dy * dy <= r_sq
                && !bitmask::get(inside, index!(im, x, y))
            {
                let q = index!(im, x, y);
                let r = (point.0 as f32 - x as f32, point.1 as f32 - y as f32);
                let dist = dot_product!(r, r).sqrt();
                let dst = 1.0 / (dist * dist.sqrt());
                let mut dir = dot_product!(r, grad_t);

                if dir.abs() <= 0.01 {
                    dir = 0.000001;
                }

                let lev = 1.0 / (1.0 + (times[q] - times[p]).abs());
                let w = (dir * dst * lev).abs();
                total += w;

                sum[0] += w * im.buf[q * 3] as f32;
                sum[1] += w * im.buf[q * 3 + 1] as f32;
                sum[2] += w * im.buf[q * 3 + 2] as f32;
            }
            x += 1; // use x += 2 to speed up more
        }
        y += 1;
    }

    *elem!(im, point.0, point.1, 0) = (sum[0] / total).ceil() as u8;
    *elem!(im, point.0, point.1, 1) = (sum[1] / total).ceil() as u8;
    *elem!(im, point.0, point.1, 2) = (sum[2] / total).ceil() as u8;
}

#[inline]
fn solve(inside: &[u32], times: &[f32], p: usize, q: usize) -> f32 {
    let t1 = times[p];
    let t2 = times[q];
    let t_min = min2!(t1, t2);

    if !bitmask::get(inside, p) {
        if !bitmask::get(inside, q) {
            if (t1 - t2).abs() >= 1.0 {
                1.0 + t_min
            } else {
                let diff = t1 - t2;
                (t1 + t2 + (2.0 - diff * diff).sqrt()) * 0.5
            }
        } else {
            1.0 + t1
        }
    } else if !bitmask::get(inside, q) {
        1.0 + t2
    } else {
        1.0 + t_min
    }
}

#[inline]
#[allow(clippy::many_single_char_names)]
fn grad_t(im: &Image, inside: &[u32], times: &[f32], p: usize) -> (f32, f32) {
    let x = p % im.width as usize;
    let y = p / im.width as usize;

    let i = if x + 1 < im.width as usize && !bitmask::get(inside, index!(im, x + 1, y)) {
        if x > 0 && !bitmask::get(inside, index!(im, x - 1, y)) {
            (times[index!(im, x + 1, y)] - times[index!(im, x - 1, y)]) * 0.5
        } else {
            times[index!(im, x + 1, y)] - times[p]
        }
    } else if x > 0 && !bitmask::get(inside, index!(im, x - 1, y)) {
        times[p] - times[index!(im, x - 1, y)]
    } else {
        0.0
    };

    let j = if y + 1 < im.height as usize && !bitmask::get(inside, index!(im, x, y + 1)) {
        if y > 0 && !bitmask::get(inside, index!(im, x, y - 1)) {
            (times[index!(im, x, y + 1)] - times[index!(im, x, y - 1)]) * 0.5
        } else {
            times[index!(im, x, y + 1)] - times[p]
        }
    } else if y > 0 && !bitmask::get(inside, index!(im, x, y - 1)) {
        times[p] - times[index!(im, x, y - 1)]
    } else {
        0.0
    };

    (i, j)
}
