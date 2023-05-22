#[repr(transparent)]
#[derive(Debug)]
pub struct XorShift32 {
    state: u32,
}

impl XorShift32 {
    pub fn new(seed: u32) -> XorShift32 {
        XorShift32 { state: seed }
    }

    pub fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "bruteforce test that needs 4GB of RAM and ~100 seconds to run the optimized build on my machine"]
    fn maximal() {
        let mut lfsr = XorShift32::new(0xDEAD);

        // could use a bitmap to cut ram usage by 8, lol
        let mut visited = vec![false; u32::MAX as usize];

        let tick = u32::MAX / 100;

        let mut count = 0;
        loop {
            let value = lfsr.next() as usize - 1;
            if visited[value] {
                break;
            }
            visited[value] = true;
            count += 1;
            if count % tick == 0 {
                println!("{}%", count / tick);
            }
        }

        assert_eq!(count, u32::MAX);
    }
}
