use logic_form::fol::Sort;
use std::{
    ffi::{c_char, c_void},
    hash::Hash,
    ptr::null_mut,
};

extern "C" {
    fn boolector_new() -> *mut c_void;
    fn boolector_set_opt(s: *mut c_void, opt: u32, val: u32);
    fn boolector_delete(s: *mut c_void);
    fn boolector_bitvec_sort(s: *mut c_void, width: u32) -> *mut c_void;
    fn boolector_release_sort(s: *mut c_void, sort: *mut c_void);
    fn boolector_var(s: *mut c_void, sort: *mut c_void, symbol: *mut c_char) -> *mut c_void;
    fn boolector_release(s: *mut c_void, term: *mut c_void);
    fn boolector_copy(s: *mut c_void, term: *mut c_void) -> *mut c_void;
    fn boolector_assume(s: *mut c_void, term: *mut c_void);
    fn boolector_sat(s: *mut c_void) -> u32;

    fn boolector_eq(s: *mut c_void, x: *mut c_void, y: *mut c_void) -> *mut c_void;
    fn boolector_ne(s: *mut c_void, x: *mut c_void, y: *mut c_void) -> *mut c_void;
}

pub struct Boolector {
    solver: *mut c_void,
}

impl Boolector {
    pub fn new() -> Self {
        let solver = unsafe { boolector_new() };
        unsafe { boolector_set_opt(solver, 1, 1) };
        Self { solver }
    }

    pub fn new_var(&mut self, sort: Sort) -> Term {
        match sort {
            Sort::BV(len) => {
                let bv_sort = self.new_bv_sort(len);
                let var =
                    unsafe { boolector_var(self.solver, bv_sort.sort, null_mut() as *mut c_char) };
                Term {
                    solver: self.solver,
                    term: var,
                }
            }
            _ => todo!(),
        }
    }

    pub fn solve(&mut self, assumps: &[Term]) -> bool {
        for a in assumps.iter() {
            unsafe { boolector_assume(self.solver, a.term) };
        }
        match unsafe { boolector_sat(self.solver) } {
            10 => true,
            20 => false,
            _ => panic!(),
        }
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

impl Drop for BtorSort {
    #[inline]
    fn drop(&mut self) {
        unsafe { boolector_release_sort(self.solver, self.sort) }
    }
}

pub struct Term {
    solver: *mut c_void,
    term: *mut c_void,
}

impl Drop for Term {
    #[inline]
    fn drop(&mut self) {
        unsafe { boolector_release(self.solver, self.term) }
    }
}

impl Clone for Term {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            solver: self.solver,
            term: unsafe { boolector_copy(self.solver, self.term) },
        }
    }
}

impl Hash for Term {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.term.hash(state);
    }
}

impl Term {
    pub fn eq(&self, other: &Self) -> Self {
        let term = unsafe { boolector_eq(self.solver, self.term, other.term) };
        Self {
            solver: self.solver,
            term,
        }
    }

    pub fn ne(&self, other: &Self) -> Self {
        let term = unsafe { boolector_ne(self.solver, self.term, other.term) };
        Self {
            solver: self.solver,
            term,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logic_form::fol::BitVec;

    #[test]
    fn it_works() {
        let mut boolector = Boolector::new();
        let bv8_x = boolector.new_var(Sort::BV(BitVec(8)));
        let bv8_y = boolector.new_var(Sort::BV(BitVec(8)));
        let eq = bv8_x.eq(&bv8_y);
        let ne = bv8_x.ne(&bv8_y);
        let res = boolector.solve(&[eq, ne]);
        dbg!(res);
    }
}
