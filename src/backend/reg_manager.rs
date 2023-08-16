use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
pub struct RegManager {
    bitmap: [AtomicBool; Self::REG_NUM],
}

impl RegManager {
    const REG_NUM: usize = 32;
    const REG_NAME: [&str; Self::REG_NUM] = [
        "x0", "ra", "sp", "gp", "tp", "t0", "t1", "t2",
        "fp", "s1", "a0", "a1", "a2", "a3", "a4", "a5",
        "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7",
        "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
    ];

    fn reg2num(reg: &str) -> usize {
        match reg {
            "x0" => 0, "ra" => 1, "sp" => 2,
            "gp" => 3, "tp" => 4, "t0" => 5,
            "t1" => 6, "t2" => 7, "fp" => 8,
            "s1" => 9, "a0" => 10, "a1" => 11,
            "a2" => 12, "a3" => 13, "a4" => 14,
            "a5" => 15, "a6" => 16, "a7" => 17,
            "s2" => 18, "s3" => 19, "s4" => 20,
            "s5" => 21, "s6" => 22, "s7" => 23,
            "s8" => 24, "s9" => 25, "s10" => 26,
            "s11" => 27, "t3" => 28, "t4" => 29,
            "t5" => 30, "t6" => 31,
            _ => unreachable!(),
        }
    }

    pub fn new() -> Self {
        const ALLOCATED: AtomicBool = AtomicBool::new(true);
        let bitmap = [ALLOCATED; Self::REG_NUM];
        for i in 5..=7 {
            bitmap[i].store(false, Relaxed);
        }
        for i in 10..=17 {
            bitmap[i].store(false, Relaxed);
        }
        for i in 28..=31 {
            bitmap[i].store(false, Relaxed);
        }
        RegManager { bitmap }
    }

    pub fn alloc(&self, reg: Option<String>) -> String {
        if let Some(reg) = reg {
            // println!("alloc: {reg}");
            let bit = &self.bitmap[Self::reg2num(&reg)];
            assert_eq!(bit.load(Relaxed), false);
            bit.store(true, Relaxed);
            return reg;
        } else {
            for i in 0..self.bitmap.len() {
                let bit = &self.bitmap[i];
                if bit.load(Relaxed) == false {
                    bit.store(true, Relaxed);
                    // println!("alloc: {}", Self::REG_NAME[i]);
                    return Self::REG_NAME[i].to_string();
                }
            }
            panic!("All registers allocated!");
        }
    }

    pub fn free(&self, reg: &str) {
        // println!("free: {reg}");
        if reg == "x0" {
            return;
        }
        let i = Self::reg2num(reg);
        assert_eq!(self.bitmap[i].load(Relaxed), true);
        self.bitmap[i].store(false, Relaxed);
    }

    #[allow(dead_code)]
    pub fn reset(&self) {
        for i in 5..=7 {
            self.bitmap[i].store(false, Relaxed);
        }
        for i in 10..=17 {
            self.bitmap[i].store(false, Relaxed);
        }
        for i in 28..=31 {
            self.bitmap[i].store(false, Relaxed);
        }
    }
}