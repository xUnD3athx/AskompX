/*
 * Askomp
 *
 * SPDX-License-Identifier: GPL-3.0-only
 * Copyright (C) 2026 UnD3ath
 */

declare name 		"Askomp";
declare version 	"1.0";
declare author 		"DSP: UnD3ath, Plugin: UnD3ath";
declare license 	"GPL-3.0-only";
declare copyright 	"Copyright (C) 2026 UnD3ath";

import("stdfaust.lib");

pos(x) = max(x, 0);
neg(x) = min(x, 0);

// The sidechain high-pass: the compressors detect on the filtered half-waves
// while the full-range signal is what actually gets gain reduced.
hp(x) = x : fi.highpass(2, sidechain_hp);

// The linear gain for the two channels of one half-wave, detector on the
// filtered signal. Same gain computer as FFcompressor_N_chan.
gain2(thresh) = co.peak_compression_gain_N_chan_db(1 - (1 / ratio),thresh,atk*0.001,rel*0.001,knee,0,stereo_link,2) : par(i,2,ba.db2linear);

// Takes the i-th bus of an n-bus signal.
take(n,i,x) = x : si.bus(n) : route(n,1,(i,1));

compress(l,r) = outl,outr with {
  gpos = (pos(hp(l)),pos(hp(r))) : gain2(postive_threshold);
  gneg = (neg(hp(l)),neg(hp(r))) : gain2(negative_threshold);
  gpl = take(2,1,gpos);
  gpr = take(2,2,gpos);
  gnl = take(2,1,gneg);
  gnr = take(2,2,gneg);
  outl = pos(l) * gpl + neg(l) * gnl;
  outr = pos(r) * gpr + neg(r) * gnr;
};

gain2(thresh) = co.peak_compression_gain_N_chan_db(1 - (1 / ratio),thresh,atk*0.001,rel*0.001,knee,0,stereo_link,2) : par(i,2,ba.db2linear);

postive_threshold = vslider("Positive Threshold",0,-36,0,0.1);
negative_threshold = vslider("Negative Threshold",0,-36,0,0.1);
atk = hslider("Attack",10,0,50,0.1);
rel = hslider("Release",50,20,200,1);
ratio = hslider("Raio",2,1,20,0.1);
knee = hslider("Knee",3,0,24,0.1);
stereo_link = hslider("Stereo Link",0,0,1,0.01);
sidechain_hp = hslider("Sidechain HP",20,20,500,1);
trim = hslider("Trim",0,-9,9,0.1);

process(l,r) = compress(l,r) : 
_*ba.db2linear(trim),_*ba.db2linear(trim);