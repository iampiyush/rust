// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// xfail-test (#3148)

struct cell<T> {
    value: T;
}

struct cells<T> {
    vals: ~[Option<cell<T>>];
}

impl<T> &cells<T> {
    fn get(idx: uint) -> &self/T {
        match self.vals[idx] {
          Some(ref v) => &v.value,
          None => fail
        }
    }
}

fn main() {}
