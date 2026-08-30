#!/usr/bin/env bash
# Regenerates src/faust_dsp.rs from dsp/askomp_dsp.dsp and applies the minimal
# patches that make the generated file compile standalone:
#   1. Type aliases for the runtime types the generated code expects.
#   2. `pub` on the mydsp fields (the dsp adapter writes the slider fields).
#   3. Removes the faust-runtime-only parts: metadata/UI builders, the
#      get_param/set_param accessors and the `FaustDsp` trait implementation.
# The compute function itself is untouched.
set -e
cd "$(dirname "$0")/.."

FAUST="${FAUST:-/c/Program Files/Faust/bin/faust}"

"$FAUST" -lang rust -double dsp/askomp_dsp.dsp -o src/faust_dsp.rs

FILE=src/faust_dsp.rs
sed -i \
    -e '/pub fn metadata(/,/^	}$/d' \
    -e '/pub fn build_user_interface(/,/^	}$/d' \
    -e '/pub fn build_user_interface_static(/,/^	}$/d' \
    -e '/pub fn get_param(/,/^	}$/d' \
    -e '/pub fn set_param(/,/^	}$/d' \
    -e '/^impl FaustDsp/,$d' \
    "$FILE"

sed -i '1i #![allow(dead_code, unused_variables, unused_mut, unused_assignments, unused_parens, non_snake_case)]\ntype FaustFloat = f64;\ntype F64 = f64;\n' "$FILE"
sed -i 's/^\t\(f[A-Za-z0-9]*\): /\tpub \1: /' "$FILE"

echo "Regenerated $FILE"
