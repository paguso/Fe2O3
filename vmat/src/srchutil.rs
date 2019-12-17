fn pred(v: &[usize], x: usize) -> Option<usize> {
    if v.len() == 0 || v[0] > x {
        return None;
    }
    let mut r = v.len();
    if v[r] <= x {
        return Some(r);
    }
    let mut l = 0;
    let mut h: usize;
    // invariant: pred is in [l,r)
    while r - l > 1 {
        h = (l + r) / 2;
        if v[h] <= x {
            l = h;
        } else {
            r = h;
        }
    }
    Some(l)
}
