use std::collections::HashMap;
use koopa::ir::entities::Value;
use super::riscv::Reg;

pub struct RegManager {
    reg2val: [Option<Value>; Self::REG_NUM],
    val2reg: HashMap<Value, Reg>,
}

impl RegManager {
    const REG_NUM: usize = 32;
    const REG_NAME: [Reg; Self::REG_NUM] = [
        "x0", "ra", "sp", "gp", "tp", "t0", "t1", "t2",
        "fp", "s1", "a0", "a1", "a2", "a3", "a4", "a5",
        "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7",
        "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
    ];
    // Hope this is fine. Koopa library didn't provide a constructor.
    const NULL_VAL: Value = unsafe { std::mem::transmute(u32::MAX) };

    fn reg2num(reg: Reg) -> usize {
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
        let mut arr = [Some(Self::NULL_VAL); Self::REG_NUM];
        for i in 6..=7 {
            arr[i] = None;
        }
        for i in 9..=31 {
            arr[i] = None;
        }
        RegManager {
            reg2val: arr,
            val2reg: HashMap::new()
        }
    }

    // When `reg` is provided but not equal to the returned register `res`,
    // it means a move from `reg` to `res` happened.
    pub fn alloc(&mut self, val: Value, reg: Option<Reg>) -> Reg {
        println!("alloc: {val:?} to {reg:?}\n");
        assert!(self.val2reg.get(&val) == None, "Value already allocated");
        if let Some(reg) = reg { // Required register is specified.
            if reg == "x0" {
                return "x0";
            }
            let idx = Self::reg2num(reg);
            assert_eq!(self.reg2val[idx], None, "Register already allocated");
            self.reg2val[idx] = Some(val);
            self.val2reg.insert(val, reg);
            return reg;
        } else { // Any register is ok.
            let f = |i: usize| {
                if self.reg2val[i] == None {
                    Some(i)
                } else {
                    None
                }
            };

            let mut range1 = 6..=7; // t1, t2
            let mut range2 = 9..=9; // s1
            let mut range3 = 18..=31; // s2-s11, t3-t6

            let idx = range1.find_map(f).or_else(|| {
                range2.find_map(f).or_else(|| {
                    range3.find_map(f)
                })
            }).expect("All registers allocated");
            self.reg2val[idx] = Some(val);
            self.val2reg.insert(val, Self::REG_NAME[idx]);
            return Self::REG_NAME[idx];
        }
    }

    // The return value indicates if swap happens.
    pub fn move_to(&mut self, old: Reg, new: Reg) -> bool {
        if old == new {
            return false;
        }
        let iold = Self::reg2num(old);
        let inew = Self::reg2num(new);
        let Some(vold) = self.reg2val[iold] else {
            panic!("Move from free register");
        };
        match self.reg2val[inew] {
            None => {
                self.reg2val[inew] = Some(vold);
                self.reg2val[iold] = None;
                self.val2reg.insert(vold, new);
                false
            }
            Some(vnew) => {
                self.reg2val[inew] = Some(vold);
                self.reg2val[iold] = Some(vnew);
                self.val2reg.insert(vold, new);
                self.val2reg.insert(vnew, old);
                true
            }
        }
    }

    pub fn reg(&self, val: Value) -> Reg {
        self.val2reg.get(&val).expect("Value not associated with register")
    }

    #[allow(dead_code)]
    pub fn value(&self, reg: Reg) -> Value {
        let idx = Self::reg2num(reg);
        let Some(val) = self.reg2val[idx] else {
            panic!("Register not associated with value");
        };
        val
    }

    pub fn free(&mut self, val: Value, reg: Reg) {
        println!("free: {val:?} from {reg:?}\n");
        self.val2reg.remove(&val);
        // You may free `x0` whenever you like.
        if reg == "x0" {
            return;
        }
        let idx = Self::reg2num(reg);
        let Some(val1) = self.reg2val[idx] else {
            panic!("Double free on register {reg}");
        };
        assert_eq!(val, val1, "Free on unmatched pair");
        self.reg2val[idx] = None;
    }

    #[allow(dead_code)]
    pub fn is_free(&mut self, reg: Reg) -> bool {
        let idx = Self::reg2num(reg);
        self.reg2val[idx] == None
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Returns all the allocated registers, collected in a `Vec`.
    pub fn regs(&self) -> Vec<Reg> {
        let f = |reg:  &Reg| *reg;
        self.val2reg.values().map(f).collect()
    }
}

impl Default for RegManager {
    fn default() -> Self {
        Self::new()
    }
}