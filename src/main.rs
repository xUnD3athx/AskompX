#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use nih_plug::prelude::*;

use askompx::Askomp;

fn main() {
    nih_export_standalone::<Askomp>();
}
