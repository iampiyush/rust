// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


struct Pair { a: int, b: &int }

const x: &int = &10;

const y: &Pair = &Pair {a: 15, b: x};

fn main() {
    io::println(fmt!("x = %?", *x));
    io::println(fmt!("y = {a: %?, b: %?}", y.a, *(y.b)));
    assert *x == 10;
    assert *(y.b) == 10;
}
