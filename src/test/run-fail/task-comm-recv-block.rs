// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// error-pattern:goodfail

extern mod std;

fn goodfail() {
    task::yield();
    fail ~"goodfail";
}

fn main() {
    task::spawn(|| goodfail() );
    let po = oldcomm::Port();
    // We shouldn't be able to get past this recv since there's no
    // message available
    let i: int = oldcomm::recv(po);
    fail ~"badfail";
}
