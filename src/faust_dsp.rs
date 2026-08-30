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
	pub fConst1: F64,
	pub fRec1: [F64;3],
	pub fHslider2: FaustFloat,
	pub fConst2: F64,
	pub fHslider3: FaustFloat,
	pub fRec0: [F64;2],
	pub fHslider4: FaustFloat,
	pub fRec3: [F64;3],
	pub fRec2: [F64;2],
	pub fHslider5: FaustFloat,
	pub fVslider1: FaustFloat,
	pub fRec4: [F64;2],
	pub fRec5: [F64;2],
	pub fHslider6: FaustFloat,
}


fn mydsp_faustpower2_f(value: F64) -> F64 {
	return value * value;
}
pub const FAUST_INPUTS: usize = 2;
pub const FAUST_OUTPUTS: usize = 2;
pub const FAUST_ACTIVES: usize = 9;
pub const FAUST_PASSIVES: usize = 0;

impl mydsp {
		
	pub fn new() -> mydsp { 
		mydsp {
			fHslider0: 0.0,
			fVslider0: 0.0,
			fHslider1: 0.0,
			fSampleRate: 0,
			fConst0: 0.0,
			fConst1: 0.0,
			fRec1: [0.0;3],
			fHslider2: 0.0,
			fConst2: 0.0,
			fHslider3: 0.0,
			fRec0: [0.0;2],
			fHslider4: 0.0,
			fRec3: [0.0;3],
			fRec2: [0.0;2],
			fHslider5: 0.0,
			fVslider1: 0.0,
			fRec4: [0.0;2],
			fRec5: [0.0;2],
			fHslider6: 0.0,
		}
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = (3.0) as FaustFloat;
		self.fVslider0 = (0.0) as FaustFloat;
		self.fHslider1 = (2e+01) as FaustFloat;
		self.fHslider2 = (5e+01) as FaustFloat;
		self.fHslider3 = (1e+01) as FaustFloat;
		self.fHslider4 = (2.0) as FaustFloat;
		self.fHslider5 = (0.0) as FaustFloat;
		self.fVslider1 = (0.0) as FaustFloat;
		self.fHslider6 = (0.0) as FaustFloat;
	}
	pub fn instance_clear(&mut self) {
		for l0 in 0..3 {
			self.fRec1[l0 as usize] = 0.0;
		}
		for l1 in 0..2 {
			self.fRec0[l1 as usize] = 0.0;
		}
		for l2 in 0..3 {
			self.fRec3[l2 as usize] = 0.0;
		}
		for l3 in 0..2 {
			self.fRec2[l3 as usize] = 0.0;
		}
		for l4 in 0..2 {
			self.fRec4[l4 as usize] = 0.0;
		}
		for l5 in 0..2 {
			self.fRec5[l5 as usize] = 0.0;
		}
	}
	pub fn instance_constants(&mut self, sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
		self.fSampleRate = sample_rate;
		self.fConst0 = F64::min(1.92e+05, F64::max(1.0, (self.fSampleRate) as F64));
		self.fConst1 = 3.141592653589793 / self.fConst0;
		self.fConst2 = 1.0 / self.fConst0;
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
			6 => Some(self.fHslider1),
			5 => Some(self.fHslider2),
			0 => Some(self.fHslider3),
			4 => Some(self.fHslider4),
			7 => Some(self.fHslider5),
			8 => Some(self.fHslider6),
			2 => Some(self.fVslider0),
			3 => Some(self.fVslider1),
			_ => None,
		}
	}
	
	pub fn set_param(&mut self, param: ParamIndex, value: FaustFloat) {
		match param.0 {
			1 => { self.fHslider0 = value }
			6 => { self.fHslider1 = value }
			5 => { self.fHslider2 = value }
			0 => { self.fHslider3 = value }
			4 => { self.fHslider4 = value }
			7 => { self.fHslider5 = value }
			8 => { self.fHslider6 = value }
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
		let mut fSlow4: F64 = F64::tan(self.fConst1 * (self.fHslider1) as F64);
		let mut fSlow5: F64 = mydsp_faustpower2_f(fSlow4);
		let mut fSlow6: F64 = 2.0 * (1.0 - 1.0 / fSlow5);
		let mut fSlow7: F64 = 1.0 / fSlow4;
		let mut fSlow8: F64 = (fSlow7 + -1.414213562373095) / fSlow4 + 1.0;
		let mut fSlow9: F64 = (fSlow7 + 1.414213562373095) / fSlow4 + 1.0;
		let mut fSlow10: F64 = 1.0 / fSlow9;
		let mut fSlow11: F64 = 1.0 / (fSlow5 * fSlow9);
		let mut fSlow12: F64 = 0.001 * (self.fHslider2) as F64;
		let mut iSlow13: i32 = (F64::abs(fSlow12) < 2.220446049250313e-16) as i32;
		let mut fSlow14: F64 = (if iSlow13 != 0 {0.0} else {F64::exp(-(self.fConst2 / (if iSlow13 != 0 {1.0} else {fSlow12})))});
		let mut fSlow15: F64 = 0.001 * (self.fHslider3) as F64;
		let mut iSlow16: i32 = (F64::abs(fSlow15) < 2.220446049250313e-16) as i32;
		let mut fSlow17: F64 = (if iSlow16 != 0 {0.0} else {F64::exp(-(self.fConst2 / (if iSlow16 != 0 {1.0} else {fSlow15})))});
		let mut fSlow18: F64 = fSlow2 - fSlow1;
		let mut fSlow19: F64 = 0.5 / F64::max(2.220446049250313e-16, fSlow0);
		let mut fSlow20: F64 = 1.0 - 1.0 / (self.fHslider4) as F64;
		let mut fSlow21: F64 = (self.fHslider5) as F64;
		let mut fSlow22: F64 = (self.fVslider1) as F64;
		let mut fSlow23: F64 = fSlow22 + fSlow1;
		let mut fSlow24: F64 = fSlow22 - fSlow1;
		let mut fSlow25: F64 = F64::powf(1e+01, 0.05 * (self.fHslider6) as F64);
		let zipped_iterators = inputs0.zip(inputs1).zip(outputs0).zip(outputs1);
		for (((input0, input1), output0), output1) in zipped_iterators {
			let mut fTemp0: F64 = (*input0) as F64;
			self.fRec1[0] = fTemp0 - fSlow10 * (fSlow8 * self.fRec1[2] + fSlow6 * self.fRec1[1]);
			let mut fTemp1: F64 = fSlow11 * (self.fRec1[2] + (self.fRec1[0] - 2.0 * self.fRec1[1]));
			let mut fTemp2: F64 = F64::abs(F64::min(fTemp1, 0.0));
			let mut fTemp3: F64 = (if (fTemp2 > self.fRec0[1]) as i32 != 0 {fSlow17} else {fSlow14});
			self.fRec0[0] = fTemp2 * (1.0 - fTemp3) + self.fRec0[1] * fTemp3;
			let mut fTemp4: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, self.fRec0[0]));
			let mut iTemp5: i32 = ((fTemp4 > fSlow18) as i32) + ((fTemp4 > fSlow3) as i32);
			let mut fTemp6: F64 = fTemp4 - fSlow2;
			let mut fTemp7: F64 = fSlow20 * F64::max(0.0, (if (iTemp5 == 0) as i32 != 0 {0.0} else {(if (iTemp5 == 1) as i32 != 0 {fSlow19 * mydsp_faustpower2_f(fSlow1 + fTemp6)} else {fTemp6})}));
			let mut fTemp8: F64 = (*input1) as F64;
			self.fRec3[0] = fTemp8 - fSlow10 * (fSlow8 * self.fRec3[2] + fSlow6 * self.fRec3[1]);
			let mut fTemp9: F64 = fSlow11 * (self.fRec3[2] + (self.fRec3[0] - 2.0 * self.fRec3[1]));
			let mut fTemp10: F64 = F64::abs(F64::min(fTemp9, 0.0));
			let mut fTemp11: F64 = (if (fTemp10 > self.fRec2[1]) as i32 != 0 {fSlow17} else {fSlow14});
			self.fRec2[0] = fTemp10 * (1.0 - fTemp11) + self.fRec2[1] * fTemp11;
			let mut fTemp12: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, self.fRec2[0]));
			let mut iTemp13: i32 = ((fTemp12 > fSlow18) as i32) + ((fTemp12 > fSlow3) as i32);
			let mut fTemp14: F64 = fTemp12 - fSlow2;
			let mut fTemp15: F64 = fSlow20 * F64::max(0.0, (if (iTemp13 == 0) as i32 != 0 {0.0} else {(if (iTemp13 == 1) as i32 != 0 {fSlow19 * mydsp_faustpower2_f(fSlow1 + fTemp14)} else {fTemp14})}));
			let mut fTemp16: F64 = F64::min(-fTemp7, -fTemp15);
			let mut fTemp17: F64 = F64::abs(F64::max(fTemp1, 0.0));
			let mut fTemp18: F64 = (if (fTemp17 > self.fRec4[1]) as i32 != 0 {fSlow17} else {fSlow14});
			self.fRec4[0] = fTemp17 * (1.0 - fTemp18) + self.fRec4[1] * fTemp18;
			let mut fTemp19: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, self.fRec4[0]));
			let mut iTemp20: i32 = ((fTemp19 > fSlow24) as i32) + ((fTemp19 > fSlow23) as i32);
			let mut fTemp21: F64 = fTemp19 - fSlow22;
			let mut fTemp22: F64 = fSlow20 * F64::max(0.0, (if (iTemp20 == 0) as i32 != 0 {0.0} else {(if (iTemp20 == 1) as i32 != 0 {fSlow19 * mydsp_faustpower2_f(fSlow1 + fTemp21)} else {fTemp21})}));
			let mut fTemp23: F64 = F64::abs(F64::max(fTemp9, 0.0));
			let mut fTemp24: F64 = (if (fTemp23 > self.fRec5[1]) as i32 != 0 {fSlow17} else {fSlow14});
			self.fRec5[0] = fTemp23 * (1.0 - fTemp24) + self.fRec5[1] * fTemp24;
			let mut fTemp25: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, self.fRec5[0]));
			let mut iTemp26: i32 = ((fTemp25 > fSlow24) as i32) + ((fTemp25 > fSlow23) as i32);
			let mut fTemp27: F64 = fTemp25 - fSlow22;
			let mut fTemp28: F64 = fSlow20 * F64::max(0.0, (if (iTemp26 == 0) as i32 != 0 {0.0} else {(if (iTemp26 == 1) as i32 != 0 {fSlow19 * mydsp_faustpower2_f(fSlow1 + fTemp27)} else {fTemp27})}));
			let mut fTemp29: F64 = F64::min(-fTemp22, -fTemp28);
			*output0 = (fSlow25 * (F64::max(fTemp0, 0.0) * F64::powf(1e+01, 0.05 * (fSlow21 * (fTemp22 + fTemp29) - fTemp22)) + F64::min(fTemp0, 0.0) * F64::powf(1e+01, 0.05 * (fSlow21 * (fTemp7 + fTemp16) - fTemp7)))) as FaustFloat;
			*output1 = (fSlow25 * (F64::max(fTemp8, 0.0) * F64::powf(1e+01, 0.05 * (fSlow21 * (fTemp28 + fTemp29) - fTemp28)) + F64::min(fTemp8, 0.0) * F64::powf(1e+01, 0.05 * (fSlow21 * (fTemp15 + fTemp16) - fTemp15)))) as FaustFloat;
			self.fRec1[2] = self.fRec1[1];
			self.fRec1[1] = self.fRec1[0];
			self.fRec0[1] = self.fRec0[0];
			self.fRec3[2] = self.fRec3[1];
			self.fRec3[1] = self.fRec3[0];
			self.fRec2[1] = self.fRec2[0];
			self.fRec4[1] = self.fRec4[0];
			self.fRec5[1] = self.fRec5[0];
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

