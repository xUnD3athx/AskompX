#![allow(dead_code, unused_variables, unused_mut, unused_assignments, unused_parens, non_snake_case)]
type FaustFloat = f64;
type F64 = f64;
pub struct ParamIndex(pub i32);

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
	pub fRec1: [F64;2],
	pub fHslider4: FaustFloat,
	pub fVslider1: FaustFloat,
	pub fRec2: [F64;2],
	pub fRec3: [F64;2],
	pub fHslider5: FaustFloat,
}


fn mydsp_faustpower2_f(value: F64) -> F64 {
	return value * value;
}
pub const FAUST_INPUTS: usize = 2;
pub const FAUST_OUTPUTS: usize = 2;
pub const FAUST_ACTIVES: usize = 8;
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
			fRec1: [0.0;2],
			fHslider4: 0.0,
			fVslider1: 0.0,
			fRec2: [0.0;2],
			fRec3: [0.0;2],
			fHslider5: 0.0,
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
		self.fHslider4 = (0.0) as FaustFloat;
		self.fVslider1 = (0.0) as FaustFloat;
		self.fHslider5 = (0.0) as FaustFloat;
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
	
	
	
	pub fn get_param(&self, param: ParamIndex) -> Option<FaustFloat> {
		match param.0 {
			1 => Some(self.fHslider0),
			5 => Some(self.fHslider1),
			0 => Some(self.fHslider2),
			4 => Some(self.fHslider3),
			6 => Some(self.fHslider4),
			7 => Some(self.fHslider5),
			2 => Some(self.fVslider0),
			3 => Some(self.fVslider1),
			_ => None,
		}
	}
	
	pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
		match param.0 {
			1 => { self.fHslider0 = value }
			5 => { self.fHslider1 = value }
			0 => { self.fHslider2 = value }
			4 => { self.fHslider3 = value }
			6 => { self.fHslider4 = value }
			7 => { self.fHslider5 = value }
			2 => { self.fVslider0 = value }
			3 => { self.fVslider1 = value }
			_ => {}
		}
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
		let mut fSlow12: F64 = 1.0 - 1.0 / (self.fHslider3) as F64;
		let mut fSlow13: F64 = (self.fHslider4) as F64;
		let mut fSlow14: F64 = (self.fVslider1) as F64;
		let mut fSlow15: F64 = fSlow14 + fSlow1;
		let mut fSlow16: F64 = fSlow14 - fSlow1;
		let mut fSlow17: F64 = F64::powf(1e+01, 0.05 * (self.fHslider5) as F64);
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
			let mut fTemp7: F64 = fSlow12 * F64::max(0.0, (if (iTemp5 == 0) as i32 != 0 {0.0} else {(if (iTemp5 == 1) as i32 != 0 {fSlow11 * mydsp_faustpower2_f(fSlow1 + fTemp6)} else {fTemp6})}));
			let mut fTemp8: F64 = (*input1) as F64;
			let mut fTemp9: F64 = F64::min(fTemp8, 0.0);
			let mut fTemp10: F64 = F64::abs(fTemp9);
			let mut fTemp11: F64 = (if (fTemp10 > self.fRec1[1]) as i32 != 0 {fSlow9} else {fSlow6});
			self.fRec1[0] = fTemp10 * (1.0 - fTemp11) + self.fRec1[1] * fTemp11;
			let mut fTemp12: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, self.fRec1[0]));
			let mut iTemp13: i32 = ((fTemp12 > fSlow10) as i32) + ((fTemp12 > fSlow3) as i32);
			let mut fTemp14: F64 = fTemp12 - fSlow2;
			let mut fTemp15: F64 = fSlow12 * F64::max(0.0, (if (iTemp13 == 0) as i32 != 0 {0.0} else {(if (iTemp13 == 1) as i32 != 0 {fSlow11 * mydsp_faustpower2_f(fSlow1 + fTemp14)} else {fTemp14})}));
			let mut fTemp16: F64 = F64::min(-fTemp7, -fTemp15);
			let mut fTemp17: F64 = F64::max(fTemp0, 0.0);
			let mut fTemp18: F64 = F64::abs(fTemp17);
			let mut fTemp19: F64 = (if (fTemp18 > self.fRec2[1]) as i32 != 0 {fSlow9} else {fSlow6});
			self.fRec2[0] = fTemp18 * (1.0 - fTemp19) + self.fRec2[1] * fTemp19;
			let mut fTemp20: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, self.fRec2[0]));
			let mut iTemp21: i32 = ((fTemp20 > fSlow16) as i32) + ((fTemp20 > fSlow15) as i32);
			let mut fTemp22: F64 = fTemp20 - fSlow14;
			let mut fTemp23: F64 = fSlow12 * F64::max(0.0, (if (iTemp21 == 0) as i32 != 0 {0.0} else {(if (iTemp21 == 1) as i32 != 0 {fSlow11 * mydsp_faustpower2_f(fSlow1 + fTemp22)} else {fTemp22})}));
			let mut fTemp24: F64 = F64::max(fTemp8, 0.0);
			let mut fTemp25: F64 = F64::abs(fTemp24);
			let mut fTemp26: F64 = (if (fTemp25 > self.fRec3[1]) as i32 != 0 {fSlow9} else {fSlow6});
			self.fRec3[0] = fTemp25 * (1.0 - fTemp26) + self.fRec3[1] * fTemp26;
			let mut fTemp27: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, self.fRec3[0]));
			let mut iTemp28: i32 = ((fTemp27 > fSlow16) as i32) + ((fTemp27 > fSlow15) as i32);
			let mut fTemp29: F64 = fTemp27 - fSlow14;
			let mut fTemp30: F64 = fSlow12 * F64::max(0.0, (if (iTemp28 == 0) as i32 != 0 {0.0} else {(if (iTemp28 == 1) as i32 != 0 {fSlow11 * mydsp_faustpower2_f(fSlow1 + fTemp29)} else {fTemp29})}));
			let mut fTemp31: F64 = F64::min(-fTemp23, -fTemp30);
			*output0 = (fSlow17 * (fTemp17 * F64::powf(1e+01, 0.05 * (fSlow13 * (fTemp23 + fTemp31) - fTemp23)) + fTemp1 * F64::powf(1e+01, 0.05 * (fSlow13 * (fTemp7 + fTemp16) - fTemp7)))) as FaustFloat;
			*output1 = (fSlow17 * (fTemp24 * F64::powf(1e+01, 0.05 * (fSlow13 * (fTemp30 + fTemp31) - fTemp30)) + fTemp9 * F64::powf(1e+01, 0.05 * (fSlow13 * (fTemp15 + fTemp16) - fTemp15)))) as FaustFloat;
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

