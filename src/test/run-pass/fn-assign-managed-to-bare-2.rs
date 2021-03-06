// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

fn add(n: int) -> fn@(int) -> int {
    fn@(m: int) -> int { m + n }
}

fn main()
{
    assert add(3)(4) == 7;

    let add1 : fn@(int)->int = add(1);
    assert add1(6) == 7;

    let add2 : &(fn@(int)->int) = &add(2);
    assert (*add2)(5) == 7;

    let add3 : fn(int)->int = add(3);
    assert add3(4) == 7;
}
