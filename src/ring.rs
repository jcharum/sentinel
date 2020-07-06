pub struct RingBuf {
    vec: Vec<String>,
    capacity: usize,
    offset: usize,
}

impl RingBuf {
    pub fn new(capacity: usize) -> RingBuf {
        RingBuf{
            vec: Vec::new(),
            capacity: capacity,
            offset: 0,
        }
    }

    pub fn push(&mut self, s: &str) {
        if self.vec.len() < self.capacity {
            self.vec.push(s.to_owned());
        } else {
            self.vec[self.offset] = s.to_owned();
            self.offset += 1;
            self.offset %= self.vec.len();
        }
    }

    pub fn contents(&self) -> String {
        let mut s = String::new();
        for i in 0..self.vec.len() {
            let j = (i+self.offset) % self.vec.len();
            s.push_str(&self.vec[j]);
            s.push('\n');
        }
        s
    }

    pub fn clear(&mut self) {
        self.offset = 0;
        self.vec.truncate(0);
    }
}
