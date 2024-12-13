use bitvec::vec::BitVec;
use std::ffi::CString;
use thiserror::Error;

use crate::error::OmbakResult;

mod dut_sys;

#[derive(Debug, Error)]
pub enum DutError {
    #[error("failed to query signals")]
    Query,
    #[error("failed to run")]
    Run,
    #[error("failed to set signal {} with value {}", _0, _1)]
    Set(String, BitVec<u32>),
    #[error("failed to get signal {}", _0)]
    Get(String),
}

pub struct Dut {
    lib: dut_sys::DutLib,
}

impl Dut {
    pub fn new(lib_path: &str) -> OmbakResult<Self> {
        let lib = dut_sys::DutLib::new(lib_path)?;
        Ok(Dut { lib })
    }

    pub fn query(&self) -> OmbakResult<Vec<Signal>> {
        let mut num_of_signals: u64 = 0;
        let sig_t_ptr = self.lib.query(&mut num_of_signals as *mut u64)?;
        Ok(Self::signals_from(sig_t_ptr, num_of_signals as usize))
    }

    pub fn run(&self, duration: u64) -> OmbakResult<u64> {
        let current_time: u64 = 0;
        match self.lib.run(duration, &current_time)? {
            0 => Ok(current_time),
            _ => Err(DutError::Run.into()),
        }
    }

    pub fn set(&self, sig_name: &str, bit_vec: &BitVec<u32>) -> OmbakResult<()> {
        let c_str = CString::new(sig_name).unwrap();
        let words = bit_vec.as_raw_slice();
        match self
            .lib
            .set(c_str.as_ptr(), words.as_ptr(), words.len() as u64)?
        {
            0 => Ok(()),
            _ => Err(DutError::Set(sig_name.to_string(), bit_vec.clone()).into()),
        }
    }

    pub fn get(&self, sig_name: &str) -> OmbakResult<BitVec<u32>> {
        let sig_name_cstr = CString::new(sig_name).unwrap();
        let mut n_bits: u64 = 0;
        let words_ptr = self
            .lib
            .get(sig_name_cstr.as_ptr(), &mut n_bits as *mut u64)?;
        if words_ptr == std::ptr::null() {
            return Err(DutError::Get(sig_name.to_string()).into());
        }
        Ok(Self::bitvec_from(words_ptr, n_bits as usize))
    }

    fn bitvec_from(words_ptr: *const u32, n_bits: usize) -> BitVec<u32> {
        let num_of_words = n_bits / 32 + if n_bits % 32 != 0 { 1 } else { 0 };
        let slice = unsafe { std::slice::from_raw_parts(words_ptr, num_of_words) };
        let mut bit_vec = BitVec::from_slice(slice);
        bit_vec.truncate(n_bits);
        bit_vec
    }

    fn signals_from(sig_t_ptr: *const dut_sys::SigT, num_of_signals: usize) -> Vec<Signal> {
        let sig_t_slice = unsafe { std::slice::from_raw_parts(sig_t_ptr, num_of_signals as usize) };
        sig_t_slice.iter().map(|s| Signal::from(s)).collect()
    }
}

#[derive(Debug)]
pub struct Signal {
    name: String,
    width: u64,
    get: bool,
    set: bool,
}