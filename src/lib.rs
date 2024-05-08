#![feature(cstr_bytes)]

use logic_form::fol::{BiOpType, ExtOpType, Sort, Term, TriOpType, UniOpType};
use std::{
    ffi::{c_char, c_void, CStr, CString},
    fmt::{self, Debug},
    hash::Hash,
};

extern "C" {
    fn boolector_new() -> *mut c_void;
    fn boolector_set_opt(s: *mut c_void, opt: u32, val: u32);
    fn boolector_delete(s: *mut c_void);
    fn boolector_bitvec_sort(s: *mut c_void, width: u32) -> *mut c_void;
    fn boolector_get_sort(s: *mut c_void, term: *mut c_void) -> *mut c_void;
    fn boolector_is_bitvec_sort(s: *mut c_void, sort: *mut c_void) -> bool;
    fn boolector_bitvec_sort_get_width(s: *mut c_void, sort: *mut c_void) -> u32;
    fn boolector_release_sort(s: *mut c_void, sort: *mut c_void);
    fn boolector_var(s: *mut c_void, sort: *mut c_void, symbol: *mut c_char) -> *mut c_void;
    fn boolector_release(s: *mut c_void, term: *mut c_void);
    fn boolector_copy(s: *mut c_void, term: *mut c_void) -> *mut c_void;
    fn boolector_get_symbol(s: *mut c_void, term: *mut c_void) -> *mut c_char;
    fn boolector_true(s: *mut c_void) -> *mut c_void;
    fn boolector_false(s: *mut c_void) -> *mut c_void;
    fn boolector_const(s: *mut c_void, c: *mut c_char) -> *mut c_void;
    fn boolector_is_const(s: *mut c_void, term: *mut c_void) -> bool;
    fn boolector_not(s: *mut c_void, x: *mut c_void) -> *mut c_void;

    fn boolector_eq(s: *mut c_void, x: *mut c_void, y: *mut c_void) -> *mut c_void;
    fn boolector_ne(s: *mut c_void, x: *mut c_void, y: *mut c_void) -> *mut c_void;
    fn boolector_and(s: *mut c_void, x: *mut c_void, y: *mut c_void) -> *mut c_void;
    fn boolector_add(s: *mut c_void, x: *mut c_void, y: *mut c_void) -> *mut c_void;
    fn boolector_ult(s: *mut c_void, x: *mut c_void, y: *mut c_void) -> *mut c_void;

    fn boolector_cond(
        s: *mut c_void,
        x: *mut c_void,
        y: *mut c_void,
        z: *mut c_void,
    ) -> *mut c_void;

    fn boolector_uext(s: *mut c_void, x: *mut c_void, width: u32) -> *mut c_void;
    fn boolector_sext(s: *mut c_void, x: *mut c_void, width: u32) -> *mut c_void;

    fn boolector_assert(s: *mut c_void, term: *mut c_void);
    fn boolector_sat(s: *mut c_void) -> u32;
    fn boolector_assume(s: *mut c_void, term: *mut c_void);
    fn boolector_bv_assignment(s: *mut c_void, term: *mut c_void) -> *mut c_char;
    fn boolector_free_bv_assignment(s: *mut c_void, a: *mut c_char);
    fn boolector_failed(s: *mut c_void, term: *mut c_void) -> bool;

}

pub struct Boolector {
    solver: *mut c_void,
}

impl Boolector {
    pub fn new() -> Self {
        let solver = unsafe { boolector_new() };
        unsafe { boolector_set_opt(solver, 0, 1) };
        unsafe { boolector_set_opt(solver, 1, 1) };
        Self { solver }
    }

    pub fn new_var(&mut self, sort: Sort, id: u32) -> BtorTerm {
        let len = match sort {
            Sort::BV(len) => len,
            Sort::Bool => 1,
        };
        let bv_sort = self.new_bv_sort(len);
        let symbol = format!("var {id}");
        let symbol = CString::new(symbol).unwrap();
        let var = unsafe { boolector_var(self.solver, bv_sort.sort, symbol.as_ptr() as _) };
        BtorTerm {
            solver: self.solver,
            term: var,
        }
    }

    pub fn bool_const(&mut self, v: bool) -> BtorTerm {
        let term = if v {
            unsafe { boolector_true(self.solver) }
        } else {
            unsafe { boolector_false(self.solver) }
        };
        BtorTerm {
            solver: self.solver,
            term,
        }
    }

    pub fn bv_const(&mut self, c: &[bool]) -> BtorTerm {
        let mut cs = String::new();
        for i in c.iter().rev() {
            cs.push(if *i { '1' } else { '0' })
        }
        let cs = CString::new(cs).unwrap();
        let term = unsafe { boolector_const(self.solver, cs.as_ptr() as _) };
        BtorTerm {
            solver: self.solver,
            term,
        }
    }

    pub fn assert(&mut self, term: &BtorTerm) {
        unsafe { boolector_assert(self.solver, term.term) };
    }

    pub fn solve(&mut self, assumps: &[BtorTerm]) -> bool {
        for a in assumps.iter() {
            unsafe { boolector_assume(self.solver, a.term) };
        }
        match unsafe { boolector_sat(self.solver) } {
            10 => true,
            20 => false,
            _ => panic!(),
        }
    }

    pub fn value(&self, term: &BtorTerm) -> Term {
        let value = unsafe { boolector_bv_assignment(self.solver, term.term) };
        let cstr = unsafe { CStr::from_ptr(value) };
        let mut res = Vec::new();
        for c in cstr.bytes().into_iter() {
            res.push(c == '1' as u8)
        }
        res.reverse();
        unsafe { boolector_free_bv_assignment(self.solver, value) };
        Term::bv_const(&res)
    }

    pub fn failed(&self, term: &BtorTerm) -> bool {
        unsafe { boolector_failed(self.solver, term.term) }
    }
}

impl Boolector {
    fn new_bv_sort(&mut self, width: u32) -> BtorSort {
        BtorSort {
            solver: self.solver,
            sort: unsafe { boolector_bitvec_sort(self.solver, width) },
        }
    }
}

impl Drop for Boolector {
    fn drop(&mut self) {
        unsafe { boolector_delete(self.solver) }
    }
}

struct BtorSort {
    solver: *mut c_void,
    sort: *mut c_void,
}

impl Into<Sort> for &BtorSort {
    fn into(self) -> Sort {
        if unsafe { boolector_is_bitvec_sort(self.solver, self.sort) } {
            let w = unsafe { boolector_bitvec_sort_get_width(self.solver, self.sort) };
            if w == 1 {
                Sort::Bool
            } else {
                Sort::BV(w)
            }
        } else {
            todo!()
        }
    }
}

impl Drop for BtorSort {
    #[inline]
    fn drop(&mut self) {
        unsafe { boolector_release_sort(self.solver, self.sort) }
    }
}

pub struct BtorTerm {
    pub solver: *mut c_void,
    pub term: *mut c_void,
}

impl Drop for BtorTerm {
    #[inline]
    fn drop(&mut self) {
        unsafe { boolector_release(self.solver, self.term) }
    }
}

impl Clone for BtorTerm {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            solver: self.solver,
            term: unsafe { boolector_copy(self.solver, self.term) },
        }
    }
}

impl PartialEq for BtorTerm {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.term == other.term
    }
}

impl Eq for BtorTerm {}

impl Hash for BtorTerm {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.term.hash(state);
    }
}

impl Debug for BtorTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = unsafe { boolector_get_symbol(self.solver, self.term) };
        if !symbol.is_null() {
            let symbol = unsafe { CStr::from_ptr(symbol) };
            dbg!(symbol);
        }
        f.debug_struct("BtorTerm")
            .field("solver", &self.solver)
            .field("term", &self.term)
            .finish()
    }
}

impl Into<Term> for &BtorTerm {
    fn into(self) -> Term {
        // let sort = self.sort();

        todo!()
    }
}

impl BtorTerm {
    pub fn sort(&self) -> Sort {
        let sort = unsafe { boolector_get_sort(self.solver, self.term) };
        let sort = BtorSort {
            solver: self.solver,
            sort,
        };
        Into::<Sort>::into(&sort)
    }

    pub fn uniop(&self, op: UniOpType) -> Self {
        let op = match op {
            UniOpType::Not => boolector_not,
            UniOpType::Inc => todo!(),
            UniOpType::Dec => todo!(),
            UniOpType::Neg => todo!(),
        };
        let term = unsafe { op(self.solver, self.term) };
        Self {
            solver: self.solver,
            term,
        }
    }

    pub fn biop(&self, other: &Self, op: BiOpType) -> Self {
        let op = match op {
            BiOpType::Iff => todo!(),
            BiOpType::Implies => todo!(),
            BiOpType::Eq => boolector_eq,
            BiOpType::Neq => boolector_ne,
            BiOpType::Sgt => todo!(),
            BiOpType::Ugt => todo!(),
            BiOpType::Sgte => todo!(),
            BiOpType::Ugte => todo!(),
            BiOpType::Slt => todo!(),
            BiOpType::Ult => boolector_ult,
            BiOpType::Slte => todo!(),
            BiOpType::Ulte => todo!(),
            BiOpType::And => boolector_and,
            BiOpType::Nand => todo!(),
            BiOpType::Nor => todo!(),
            BiOpType::Or => todo!(),
            BiOpType::Xnor => todo!(),
            BiOpType::Xor => todo!(),
            BiOpType::Rol => todo!(),
            BiOpType::Ror => todo!(),
            BiOpType::Sll => todo!(),
            BiOpType::Sra => todo!(),
            BiOpType::Srl => todo!(),
            BiOpType::Add => boolector_add,
            BiOpType::Mul => todo!(),
            BiOpType::Sdiv => todo!(),
            BiOpType::Udiv => todo!(),
            BiOpType::Smod => todo!(),
            BiOpType::Srem => todo!(),
            BiOpType::Urem => todo!(),
            BiOpType::Sub => todo!(),
            BiOpType::Saddo => todo!(),
            BiOpType::Uaddo => todo!(),
            BiOpType::Sdivo => todo!(),
            BiOpType::Udivo => todo!(),
            BiOpType::Smulo => todo!(),
            BiOpType::Umulo => todo!(),
            BiOpType::Ssubo => todo!(),
            BiOpType::Usubo => todo!(),
            BiOpType::Concat => todo!(),
            BiOpType::Read => todo!(),
        };
        let term = unsafe { op(self.solver, self.term, other.term) };
        Self {
            solver: self.solver,
            term,
        }
    }

    pub fn eq(&self, other: &Self) -> Self {
        self.biop(other, BiOpType::Eq)
    }

    pub fn neq(&self, other: &Self) -> Self {
        self.biop(other, BiOpType::Neq)
    }

    pub fn and(&self, other: &Self) -> Self {
        self.biop(other, BiOpType::And)
    }

    pub fn add(&self, other: &Self) -> Self {
        self.biop(other, BiOpType::Add)
    }

    pub fn triop(&self, x: &Self, y: &Self, op: TriOpType) -> Self {
        let op = match op {
            TriOpType::Ite => boolector_cond,
            TriOpType::Write => todo!(),
        };
        let term = unsafe { op(self.solver, self.term, x.term, y.term) };
        Self {
            solver: self.solver,
            term,
        }
    }

    #[inline]
    pub fn extop(&self, op: ExtOpType, length: u32) -> Self {
        let op = match op {
            ExtOpType::Sext => boolector_sext,
            ExtOpType::Uext => boolector_uext,
        };
        let term = unsafe { op(self.solver, self.term, length) };
        Self {
            solver: self.solver,
            term,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut boolector = Boolector::new();
        let bv8_x = boolector.new_var(Sort::BV(8), 0);
        let bv8_y = boolector.new_var(Sort::BV(8), 1);
        let eq = bv8_x.eq(&bv8_y);
        let neq = bv8_x.neq(&bv8_y);
        let mut t = boolector.bool_const(true);
        t = neq.and(&t);
        let res = boolector.solve(&[eq, neq]);
        dbg!(res);
        dbg!(t);
    }
}
