/// A forward-only cursor to read values from an RLE [count, value] stream
/// in *decoded index* order. Works in O(1) amortized for monotonically
/// increasing target indices.
#[derive(Clone, Copy)]
pub struct RleCursor<'a> {
    buf: &'a [u8],
    i: usize,
    run_c: usize,
    val: u8,
    dec: usize,
}

impl<'a> RleCursor<'a> {
    #[inline(always)]
    pub fn new(buf: &'a [u8]) -> Self {
        let mut c = Self {
            buf,
            i: 0,
            run_c: 0,
            val: 0,
            dec: 0,
        };
        c.load_next_run();
        c
    }

    #[inline(always)]
    fn load_next_run(&mut self) {
        if self.i + 1 < self.buf.len() {
            let count = self.buf[self.i] as usize;
            let value = self.buf[self.i + 1];
            self.dec += self.run_c;
            self.i += 2;
            self.run_c = count;
            self.val = value;
        } else {
            // Exhausted stream
            self.dec += self.run_c;
            self.run_c = 0;
            self.val = 0;
            self.i = self.buf.len();
        }
    }

    /// Advance forward until the run that *contains* `target_dec_idx`.
    /// `target_dec_idx` must be >= current decoded index for best performance.
    #[inline(always)]
    pub fn advance_to(&mut self, target_dec_idx: usize) {
        // If target is before current run start, we can't go backwards (shouldn't happen
        // in our monotonic usage). We'll just fall back to full rescan if it occurs.
        if target_dec_idx < self.dec {
            // Rare/unsafe path: rescan from the beginning (still O(N), but should not happen).
            *self = RleCursor::new(self.buf);
        }
        // Move runs forward until target is inside [dec .. dec + run_c)
        while self.run_c == 0 || target_dec_idx >= self.dec + self.run_c {
            if self.run_c == 0 && self.i >= self.buf.len() {
                // End of stream
                return;
            }
            self.load_next_run();
        }
        // Now target lies in current run
    }

    /// Get the value at `target_dec_idx`, advancing forward as needed.
    #[inline(always)]
    pub fn get(&mut self, target_dec_idx: usize) -> u8 {
        self.advance_to(target_dec_idx);
        self.val
    }
}

#[cfg(test)]
mod tests {
    use super::RleCursor;

    #[test]
    fn single_run_returns_same_value() {
        let buf = [3u8, 42];
        let mut cur = RleCursor::new(&buf);

        assert_eq!(cur.get(0), 42);
        assert_eq!(cur.get(1), 42);
        assert_eq!(cur.get(2), 42);
    }

    #[test]
    fn multiple_runs_decode_correctly() {
        let buf = [2, 10, 3, 20];
        let mut cur = RleCursor::new(&buf);

        assert_eq!(cur.get(0), 10);
        assert_eq!(cur.get(1), 10);
        assert_eq!(cur.get(2), 20);
        assert_eq!(cur.get(3), 20);
        assert_eq!(cur.get(4), 20);
    }

    #[test]
    fn monotonic_access_advances_cursor_once() {
        let buf = [1, 1, 1, 2, 1, 3, 1, 4];
        let mut cur = RleCursor::new(&buf);

        for i in 0..4 {
            assert_eq!(cur.get(i), (i + 1) as u8);
        }
    }

    #[test]
    fn non_monotonic_access_rescans_safely() {
        let buf = [3, 7, 2, 9];
        let mut cur = RleCursor::new(&buf);

        assert_eq!(cur.get(0), 7);
        assert_eq!(cur.get(3), 9);
        assert_eq!(cur.get(1), 7);
    }

    #[test]
    fn end_of_stream_returns_zero() {
        let buf = [2, 5];
        let mut cur = RleCursor::new(&buf);

        assert_eq!(cur.get(0), 5);
        assert_eq!(cur.get(1), 5);
        assert_eq!(cur.get(2), 0);
    }

    #[test]
    fn empty_stream_always_returns_zero() {
        let buf: [u8; 0] = [];
        let mut cur = RleCursor::new(&buf);

        assert_eq!(cur.get(0), 0);
        assert_eq!(cur.get(10), 0);
    }
}
