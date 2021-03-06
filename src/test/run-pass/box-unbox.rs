// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.



struct Box<T: Copy> {c: @T}

fn unbox<T: Copy>(b: Box<T>) -> T { return *b.c; }

fn main() {
    let foo: int = 17;
    let bfoo: Box<int> = Box {c: @foo};
    debug!("see what's in our box");
    assert (unbox::<int>(bfoo) == foo);
}
