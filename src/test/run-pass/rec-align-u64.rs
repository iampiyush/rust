// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Issue #2303

#[abi = "rust-intrinsic"]
extern mod rusti {
    #[legacy_exports];
    fn pref_align_of<T>() -> uint;
    fn min_align_of<T>() -> uint;
}

// This is the type with the questionable alignment
struct Inner {
    c64: u64
}

// This is the type that contains the type with the
// questionable alignment, for testing
struct Outer {
    c8: u8,
    t: Inner
}


#[cfg(target_os = "linux")]
#[cfg(target_os = "macos")]
#[cfg(target_os = "freebsd")]
mod m {
    #[legacy_exports];
    #[cfg(target_arch = "x86")]
    mod m {
        #[legacy_exports];
        fn align() -> uint { 4u }
        fn size() -> uint { 12u }
    }

    #[cfg(target_arch = "x86_64")]
    mod m {
        #[legacy_exports];
        fn align() -> uint { 8u }
        fn size() -> uint { 16u }
    }
}

#[cfg(target_os = "win32")]
mod m {
    #[legacy_exports];
    #[cfg(target_arch = "x86")]
    mod m {
        #[legacy_exports];
        fn align() -> uint { 8u }
        fn size() -> uint { 16u }
    }
}

fn main() {
    unsafe {
        let x = Outer {c8: 22u8, t: Inner {c64: 44u64}};

        // Send it through the shape code
        let y = fmt!("%?", x);

        debug!("align inner = %?", rusti::min_align_of::<Inner>());
        debug!("size outer = %?", sys::size_of::<Outer>());
        debug!("y = %s", y);

        // per clang/gcc the alignment of `Inner` is 4 on x86.
        assert rusti::min_align_of::<Inner>() == m::m::align();

        // per clang/gcc the size of `Outer` should be 12
        // because `Inner`s alignment was 4.
        assert sys::size_of::<Outer>() == m::m::size();

        assert y == ~"{c8: 22, t: {c64: 44}}";
    }
}
