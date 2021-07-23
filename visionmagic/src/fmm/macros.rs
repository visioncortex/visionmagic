macro_rules! min2 {
    ($x:expr, $y:expr) => {{
        if $x < $y {
            $x
        } else {
            $y
        }
    }};
}

macro_rules! min4 {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        min2!(min2!($a, $b), min2!($c, $d))
    };
}

macro_rules! dot_product {
    ($u:expr, $v:expr) => {
        $u.0 * $v.0 + $u.1 * $v.1
    };
}

macro_rules! index {
    ($im:expr, $x:expr, $y:expr) => {
        ($y as usize) * ($im.width as usize) + ($x as usize)
    };
}

macro_rules! elem {
    ($im:expr, $x:expr, $y:expr, $c:expr) => {
        unsafe { $im.buf.get_unchecked_mut(index!($im, $x, $y) * 3 + $c) }
    };
}

macro_rules! elem_v {
    ($im:expr, $x:expr, $y:expr, $c:expr) => {
        unsafe { $im.buf.get_unchecked(index!($im, $x, $y) * 3 + $c) }
    };
}
