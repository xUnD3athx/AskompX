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
	pub fHslider1: FaustFloat,
	pub fVslider0: FaustFloat,
	pub fHslider2: FaustFloat,
	pub fSampleRate: i32,
	pub fConst0: F64,
	pub fHslider3: FaustFloat,
	pub fRec1: [F64;2],
	pub fHslider4: FaustFloat,
	pub fRec0: [F64;2],
	pub fVslider1: FaustFloat,
	pub fRec3: [F64;2],
	pub fRec2: [F64;2],
	pub fHslider5: FaustFloat,
	pub fRec5: [F64;2],
	pub fRec4: [F64;2],
	pub fRec7: [F64;2],
	pub fRec6: [F64;2],
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
			fHslider1: 0.0,
			fVslider0: 0.0,
			fHslider2: 0.0,
			fSampleRate: 0,
			fConst0: 0.0,
			fHslider3: 0.0,
			fRec1: [0.0;2],
			fHslider4: 0.0,
			fRec0: [0.0;2],
			fVslider1: 0.0,
			fRec3: [0.0;2],
			fRec2: [0.0;2],
			fHslider5: 0.0,
			fRec5: [0.0;2],
			fRec4: [0.0;2],
			fRec7: [0.0;2],
			fRec6: [0.0;2],
		}
	}

	pub fn get_sample_rate(&self) -> i32 { self.fSampleRate as i32}
	
	pub fn class_init(sample_rate: i32) {
		// Obtaining locks on 0 static var(s)
	}
	pub fn instance_reset_params(&mut self) {
		self.fHslider0 = (0.0) as FaustFloat;
		self.fHslider1 = (3.0) as FaustFloat;
		self.fVslider0 = (0.0) as FaustFloat;
		self.fHslider2 = (5e+01) as FaustFloat;
		self.fHslider3 = (1e+01) as FaustFloat;
		self.fHslider4 = (2.0) as FaustFloat;
		self.fVslider1 = (0.0) as FaustFloat;
		self.fHslider5 = (0.0) as FaustFloat;
	}
	pub fn instance_clear(&mut self) {
		for l0 in 0..2 {
			self.fRec1[l0 as usize] = 0.0;
		}
		for l1 in 0..2 {
			self.fRec0[l1 as usize] = 0.0;
		}
		for l2 in 0..2 {
			self.fRec3[l2 as usize] = 0.0;
		}
		for l3 in 0..2 {
			self.fRec2[l3 as usize] = 0.0;
		}
		for l4 in 0..2 {
			self.fRec5[l4 as usize] = 0.0;
		}
		for l5 in 0..2 {
			self.fRec4[l5 as usize] = 0.0;
		}
		for l6 in 0..2 {
			self.fRec7[l6 as usize] = 0.0;
		}
		for l7 in 0..2 {
			self.fRec6[l7 as usize] = 0.0;
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
		let mut iSlow1: i32 = (fSlow0 != 1.0) as i32;
		let mut fSlow2: F64 = (self.fHslider1) as F64;
		let mut fSlow3: F64 = 0.5 * fSlow2;
		let mut fSlow4: F64 = (self.fVslider0) as F64;
		let mut fSlow5: F64 = fSlow4 + fSlow3;
		let mut iSlow6: i32 = (fSlow0) as i32;
		let mut fSlow7: F64 = 0.001 * (self.fHslider2) as F64;
		let mut iSlow8: i32 = (F64::abs(fSlow7) < 2.220446049250313e-16) as i32;
		let mut fSlow9: F64 = (if iSlow8 != 0 {0.0} else {F64::exp(-(self.fConst0 / (if iSlow8 != 0 {1.0} else {fSlow7})))});
		let mut fSlow10: F64 = 0.001 * (self.fHslider3) as F64;
		let mut iSlow11: i32 = (F64::abs(fSlow10) < 2.220446049250313e-16) as i32;
		let mut fSlow12: F64 = (if iSlow11 != 0 {0.0} else {F64::exp(-(self.fConst0 / (if iSlow11 != 0 {1.0} else {fSlow10})))});
		let mut fSlow13: F64 = fSlow4 - fSlow3;
		let mut fSlow14: F64 = 0.5 / F64::max(2.220446049250313e-16, fSlow2);
		let mut fSlow15: F64 = 1.0 - 1.0 / (self.fHslider4) as F64;
		let mut fSlow16: F64 = (self.fVslider1) as F64;
		let mut fSlow17: F64 = fSlow16 + fSlow3;
		let mut fSlow18: F64 = fSlow16 - fSlow3;
		let mut fSlow19: F64 = F64::powf(1e+01, 0.05 * (self.fHslider5) as F64);
		let zipped_iterators = inputs0.zip(inputs1).zip(outputs0).zip(outputs1);
		for (((input0, input1), output0), output1) in zipped_iterators {
			let mut fTemp0: F64 = (*input0) as F64;
			let mut fTemp1: F64 = F64::min(fTemp0, 0.0);
			let mut fTemp2: F64 = F64::abs(fTemp1);
			let mut fTemp3: F64 = (if iSlow6 != 0 {0.0} else {fTemp2});
			let mut fTemp4: F64 = (if (fTemp3 > self.fRec1[1]) as i32 != 0 {fSlow12} else {fSlow9});
			self.fRec1[0] = fTemp3 * (1.0 - fTemp4) + self.fRec1[1] * fTemp4;
			let mut fTemp5: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, (if iSlow6 != 0 {fTemp2} else {self.fRec1[0]})));
			let mut iTemp6: i32 = ((fTemp5 > fSlow13) as i32) + ((fTemp5 > fSlow5) as i32);
			let mut fTemp7: F64 = fTemp5 - fSlow4;
			let mut fTemp8: F64 = fSlow15 * F64::max(0.0, (if (iTemp6 == 0) as i32 != 0 {0.0} else {(if (iTemp6 == 1) as i32 != 0 {fSlow14 * mydsp_faustpower2_f(fSlow3 + fTemp7)} else {fTemp7})}));
			let mut fTemp9: F64 = (if iSlow1 != 0 {0.0} else {-fTemp8});
			let mut fTemp10: F64 = (if (fTemp9 > self.fRec0[1]) as i32 != 0 {fSlow9} else {fSlow12});
			self.fRec0[0] = fTemp9 * (1.0 - fTemp10) + self.fRec0[1] * fTemp10;
			let mut fTemp11: F64 = F64::max(fTemp0, 0.0);
			let mut fTemp12: F64 = F64::abs(fTemp11);
			let mut fTemp13: F64 = (if iSlow6 != 0 {0.0} else {fTemp12});
			let mut fTemp14: F64 = (if (fTemp13 > self.fRec3[1]) as i32 != 0 {fSlow12} else {fSlow9});
			self.fRec3[0] = fTemp13 * (1.0 - fTemp14) + self.fRec3[1] * fTemp14;
			let mut fTemp15: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, (if iSlow6 != 0 {fTemp12} else {self.fRec3[0]})));
			let mut iTemp16: i32 = ((fTemp15 > fSlow18) as i32) + ((fTemp15 > fSlow17) as i32);
			let mut fTemp17: F64 = fTemp15 - fSlow16;
			let mut fTemp18: F64 = fSlow15 * F64::max(0.0, (if (iTemp16 == 0) as i32 != 0 {0.0} else {(if (iTemp16 == 1) as i32 != 0 {fSlow14 * mydsp_faustpower2_f(fSlow3 + fTemp17)} else {fTemp17})}));
			let mut fTemp19: F64 = (if iSlow1 != 0 {0.0} else {-fTemp18});
			let mut fTemp20: F64 = (if (fTemp19 > self.fRec2[1]) as i32 != 0 {fSlow9} else {fSlow12});
			self.fRec2[0] = fTemp19 * (1.0 - fTemp20) + self.fRec2[1] * fTemp20;
			*output0 = (fSlow19 * (fTemp11 * F64::powf(1e+01, 0.05 * (if iSlow1 != 0 {-fTemp18} else {self.fRec2[0]})) + fTemp1 * F64::powf(1e+01, 0.05 * (if iSlow1 != 0 {-fTemp8} else {self.fRec0[0]})))) as FaustFloat;
			let mut fTemp21: F64 = (*input1) as F64;
			let mut fTemp22: F64 = F64::min(fTemp21, 0.0);
			let mut fTemp23: F64 = F64::abs(fTemp22);
			let mut fTemp24: F64 = (if iSlow6 != 0 {0.0} else {fTemp23});
			let mut fTemp25: F64 = (if (fTemp24 > self.fRec5[1]) as i32 != 0 {fSlow12} else {fSlow9});
			self.fRec5[0] = fTemp24 * (1.0 - fTemp25) + self.fRec5[1] * fTemp25;
			let mut fTemp26: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, (if iSlow6 != 0 {fTemp23} else {self.fRec5[0]})));
			let mut iTemp27: i32 = ((fTemp26 > fSlow13) as i32) + ((fTemp26 > fSlow5) as i32);
			let mut fTemp28: F64 = fTemp26 - fSlow4;
			let mut fTemp29: F64 = fSlow15 * F64::max(0.0, (if (iTemp27 == 0) as i32 != 0 {0.0} else {(if (iTemp27 == 1) as i32 != 0 {fSlow14 * mydsp_faustpower2_f(fSlow3 + fTemp28)} else {fTemp28})}));
			let mut fTemp30: F64 = (if iSlow1 != 0 {0.0} else {-fTemp29});
			let mut fTemp31: F64 = (if (fTemp30 > self.fRec4[1]) as i32 != 0 {fSlow9} else {fSlow12});
			self.fRec4[0] = fTemp30 * (1.0 - fTemp31) + self.fRec4[1] * fTemp31;
			let mut fTemp32: F64 = F64::max(fTemp21, 0.0);
			let mut fTemp33: F64 = F64::abs(fTemp32);
			let mut fTemp34: F64 = (if iSlow6 != 0 {0.0} else {fTemp33});
			let mut fTemp35: F64 = (if (fTemp34 > self.fRec7[1]) as i32 != 0 {fSlow12} else {fSlow9});
			self.fRec7[0] = fTemp34 * (1.0 - fTemp35) + self.fRec7[1] * fTemp35;
			let mut fTemp36: F64 = 2e+01 * F64::log10(F64::max(2.2250738585072014e-308, (if iSlow6 != 0 {fTemp33} else {self.fRec7[0]})));
			let mut iTemp37: i32 = ((fTemp36 > fSlow18) as i32) + ((fTemp36 > fSlow17) as i32);
			let mut fTemp38: F64 = fTemp36 - fSlow16;
			let mut fTemp39: F64 = fSlow15 * F64::max(0.0, (if (iTemp37 == 0) as i32 != 0 {0.0} else {(if (iTemp37 == 1) as i32 != 0 {fSlow14 * mydsp_faustpower2_f(fSlow3 + fTemp38)} else {fTemp38})}));
			let mut fTemp40: F64 = (if iSlow1 != 0 {0.0} else {-fTemp39});
			let mut fTemp41: F64 = (if (fTemp40 > self.fRec6[1]) as i32 != 0 {fSlow9} else {fSlow12});
			self.fRec6[0] = fTemp40 * (1.0 - fTemp41) + self.fRec6[1] * fTemp41;
			*output1 = (fSlow19 * (fTemp32 * F64::powf(1e+01, 0.05 * (if iSlow1 != 0 {-fTemp39} else {self.fRec6[0]})) + fTemp22 * F64::powf(1e+01, 0.05 * (if iSlow1 != 0 {-fTemp29} else {self.fRec4[0]})))) as FaustFloat;
			self.fRec1[1] = self.fRec1[0];
			self.fRec0[1] = self.fRec0[0];
			self.fRec3[1] = self.fRec3[0];
			self.fRec2[1] = self.fRec2[0];
			self.fRec5[1] = self.fRec5[0];
			self.fRec4[1] = self.fRec4[0];
			self.fRec7[1] = self.fRec7[0];
			self.fRec6[1] = self.fRec6[0];
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

