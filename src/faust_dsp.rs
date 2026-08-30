#![allow(dead_code, unused_variables, unused_mut, unused_assignments, unused_parens, non_snake_case)]
type FaustFloat = f64;
type F64 = f64;

/* ------------------------------------------------------------
author: "DSP: UnD3ath, Plugin: UnD3ath"
copyright: "Copyright (C) 2026 UnD3ath / Aqua Sounds"
license: "GPL-2.0"
name: "Askomp"
version: "1.0"
Code generated with Faust 2.85.5 (https://faust.grame.fr)
Compilation options: -lang rust -fpga-mem-th 4 -ct 1 -es 1 -mcd 16 -mdd 1024 -mdy 33 -double -ftz 0
------------------------------------------------------------ */

#[repr(C)]
pub struct mydsp {
	pub fHslider0: FaustFloat,
	pub fVslider0: FaustFloat,
	pub fHslider1: FaustFloat,
	pub fSampleRate: i32,
	pub fConst0: F64,
	pub fHslider2: FaustFloat,
	pub fRec0: [F64;2],
	pub fHslider3: FaustFloat,
	pub fVslider1: FaustFloat,
	pub fRec1: [F64;2],
	pub fHslider4: FaustFloat,
	pub fRec2: [F64;2],
	pub fRec3: [F64;2],
}


fn mydsp_faustpower2_f(value: F64) -> F64 {
	return value * value;
}
pub const FAUST_INPUTS: usize = 2;
pub const FAUST_OUTPUTS: usize = 2;
pub const FAUST_ACTIVES: usize = 7;
pub const FAUST_PASSIVES: usize = 0;

impl mydsp {
		
	pub fn new() -> mydsp { 
		mydsp {
			fHslider0: 0.0,
			fVslider0: 0.0,
			fHslider1: 0.0,
			fSampleRate: 0,
			fConst0: 0.0,
			fHslider2: 0.0,
			fRec0: [0.0;2],
			fHslider3: 0.0,
			fVslider1: 0.0,
			fRec1: [0.0;2],
			fHslider4: 0.0,
			fRec2: [0.0;2],
			fRec3: [0.0;2],
		}
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = (3.0) as FaustFloat;
		self.fVslider0 = (0.0) as FaustFloat;
		self.fHslider1 = (5e+01) as FaustFloat;
		self.fHslider2 = (1e+01) as FaustFloat;
		self.fHslider3 = (2.0) as FaustFloat;
		self.fVslider1 = (0.0) as FaustFloat;
		self.fHslider4 = (0.0) as FaustFloat;
	}
	pub fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.fRec0[l0 as usize] = 0.0;
		}
		for l1 in 0..2 {
			self.fRec1[l1 as usize] = 0.0;
		}
		for l2 in 0..2 {
			self.fRec2[l2 as usize] = 0.0;
		}
		for l3 in 0..2 {
			self.fRec3[l3 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
		self.fSampleRate = sample_rate;
		self.fConst0 = 1.0 / F64::min(1.92e+05, F64::max(1.0, (self.fSampleRate) as F64));
	}
	pub fn instance_init(&mut self, sample_rate: i32) {
		self.instance_constants(sample_rate);
		self.instance_reset_params();
		self.instance_clear();
	}
	pub fn init(&mut self, sample_rate: i32) {
		mydsp::class_init(sample_rate);
		self.instance_init(sample_rate);
	}
	
	
	
	
	
	pub fn compute(
		&mut self,
		count: usize,
		inputs: &[impl AsRef<[FaustFloat]>],
		outputs: &mut[impl AsMut<[FaustFloat]>],
	) {
		
		// Obtaining locks on 0 static var(s)
		let [inputs0, inputs1, .. ] = inputs.as_ref() else { panic!("wrong number of input buffers"); };
		let inputs0 = inputs0.as_ref()[..count].iter();
		let inputs1 = inputs1.as_ref()[..count].iter();
		let [outputs0, outputs1, .. ] = outputs.as_mut() else { panic!("wrong number of output buffers"); };
		let outputs0 = outputs0.as_mut()[..count].iter_mut();
		let outputs1 = outputs1.as_mut()[..count].iter_mut();
		let mut fSlow0: F64 = (self.fHslider0) as F64;
		let mut fSlow1: F64 = 0.5 * fSlow0;
		let mut fSlow2: F64 = (self.fVslider0) as F64;
		let mut fSlow3: F64 = fSlow2 + fSlow1;
		let mut fSlow4: F64 = 0.001 * (self.fHslider1) as F64;
		let mut iSlow5: i32 = (F64::abs(fSlow4) < 2.220446049250313e-16) as i32;
		let mut fSlow6: F64 = (if iSlow5 != 0 {0.0} else {F64::exp(-(self.fConst0 / (if iSlow5 != 0 {1.0} else {fSlow4})))});
		let mut fSlow7: F64 = 0.001 * (self.fHslider2) as F64;
		let mut iSlow8: i32 = (F64::abs(fSlow7) < 2.220446049250313e-16) as i32;
		let mut fSlow9: F64 = (if iSlow8 != 0 {0.0} else {F64::exp(-(self.fConst0 / (if iSlow8 != 0 {1.0} else {fSlow7})))});
		let mut fSlow10: F64 = fSlow2 - fSlow1;
		let mut fSlow11: F64 = 0.5 / F64::max(2.220446049250313e-16, fSlow0);
		let mut fSlow12: F64 = 0.05 * (1.0 - 1.0 / (self.fHslider3) as F64);
		let mut fSlow13: F64 = (self.fVslider1) as F64;
		let mut fSlow14: F64 = fSlow13 + fSlow1;
		let mut fSlow15: F64 = fSlow13 - fSlow1;
		let mut fSlow16: F64 = F64::powf(1e+01, 0.05 * (self.fHslider4) as F64);
		let zipped_iterators = inputs0.zip(inputs1).zip(outputs0).zip(outputs1);
		for (((input0, input1), output0), output1) in zipped_iterators {
			let mut fTemp0: F64 = (*input0) as F64;
			let mut fTemp1: F64 = F64::min(fTemp0, 0.0);
			let mut fTemp2: F64 = F64::abs(fTemp1);
			let mut fTemp3: F64 = (if (fTemp2 > self.fRec0[1]) as i32 != 0 {fSlow9} else {fSlow6});
			self.fRec0[0] = fTemp2 * (1.0 - fTemp3) + self.fRec0[1] * fTemp3;
			let mut fTemp4: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, self.fRec0[0]));
			let mut iTemp5: i32 = ((fTemp4 > fSlow10) as i32) + ((fTemp4 > fSlow3) as i32);
			let mut fTemp6: F64 = fTemp4 - fSlow2;
			let mut fTemp7: F64 = F64::max(fTemp0, 0.0);
			let mut fTemp8: F64 = F64::abs(fTemp7);
			let mut fTemp9: F64 = (if (fTemp8 > self.fRec1[1]) as i32 != 0 {fSlow9} else {fSlow6});
			self.fRec1[0] = fTemp8 * (1.0 - fTemp9) + self.fRec1[1] * fTemp9;
			let mut fTemp10: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, self.fRec1[0]));
			let mut iTemp11: i32 = ((fTemp10 > fSlow15) as i32) + ((fTemp10 > fSlow14) as i32);
			let mut fTemp12: F64 = fTemp10 - fSlow13;
			*output0 = (fSlow16 * (fTemp7 * F64::powf(1e+01, -(fSlow12 * F64::max(0.0, (if (iTemp11 == 0) as i32 != 0 {0.0} else {(if (iTemp11 == 1) as i32 != 0 {fSlow11 * mydsp_faustpower2_f(fSlow1 + fTemp12)} else {fTemp12})})))) + fTemp1 * F64::powf(1e+01, -(fSlow12 * F64::max(0.0, (if (iTemp5 == 0) as i32 != 0 {0.0} else {(if (iTemp5 == 1) as i32 != 0 {fSlow11 * mydsp_faustpower2_f(fSlow1 + fTemp6)} else {fTemp6})})))))) as FaustFloat;
			let mut fTemp13: F64 = (*input1) as F64;
			let mut fTemp14: F64 = F64::min(fTemp13, 0.0);
			let mut fTemp15: F64 = F64::abs(fTemp14);
			let mut fTemp16: F64 = (if (fTemp15 > self.fRec2[1]) as i32 != 0 {fSlow9} else {fSlow6});
			self.fRec2[0] = fTemp15 * (1.0 - fTemp16) + self.fRec2[1] * fTemp16;
			let mut fTemp17: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, self.fRec2[0]));
			let mut iTemp18: i32 = ((fTemp17 > fSlow10) as i32) + ((fTemp17 > fSlow3) as i32);
			let mut fTemp19: F64 = fTemp17 - fSlow2;
			let mut fTemp20: F64 = F64::max(fTemp13, 0.0);
			let mut fTemp21: F64 = F64::abs(fTemp20);
			let mut fTemp22: F64 = (if (fTemp21 > self.fRec3[1]) as i32 != 0 {fSlow9} else {fSlow6});
			self.fRec3[0] = fTemp21 * (1.0 - fTemp22) + self.fRec3[1] * fTemp22;
			let mut fTemp23: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, self.fRec3[0]));
			let mut iTemp24: i32 = ((fTemp23 > fSlow15) as i32) + ((fTemp23 > fSlow14) as i32);
			let mut fTemp25: F64 = fTemp23 - fSlow13;
			*output1 = (fSlow16 * (fTemp20 * F64::powf(1e+01, -(fSlow12 * F64::max(0.0, (if (iTemp24 == 0) as i32 != 0 {0.0} else {(if (iTemp24 == 1) as i32 != 0 {fSlow11 * mydsp_faustpower2_f(fSlow1 + fTemp25)} else {fTemp25})})))) + fTemp14 * F64::powf(1e+01, -(fSlow12 * F64::max(0.0, (if (iTemp18 == 0) as i32 != 0 {0.0} else {(if (iTemp18 == 1) as i32 != 0 {fSlow11 * mydsp_faustpower2_f(fSlow1 + fTemp19)} else {fTemp19})})))))) as FaustFloat;
			self.fRec0[1] = self.fRec0[0];
			self.fRec1[1] = self.fRec1[0];
			self.fRec2[1] = self.fRec2[0];
			self.fRec3[1] = self.fRec3[0];
		}
		
	}

}

#[cfg(not(target_arch = "wasm32"))] // Compile ffi bindings only on non-wasm targets
	mod ffi {
	use core::ffi::c_double;
	// Conditionally compile the link attribute only on non-Windows platforms
	#[cfg_attr(not(target_os="windows"), link(name="m"))]
	unsafe extern "C" {
		pub fn remainder(from: c_double, to: c_double) -> c_double;
		pub fn rint(val: c_double) -> c_double;
	}
}
fn remainder(from: f64, to: f64) -> f64 {
	#[cfg(not(target_arch = "wasm32"))] // non-wasm targets use ffi bindings
	unsafe { ffi::remainder(from, to) }
	#[cfg(target_arch = "wasm32")] // wasm relies on libm
	libm::remainder(from, to)
}
fn rint(val: f64) -> f64 {
	#[cfg(not(target_arch = "wasm32"))] // non-wasm targets use ffi bindings
	unsafe { ffi::rint(val) }
	#[cfg(target_arch = "wasm32")] // wasm relies on libm
	libm::rint(val)
}

