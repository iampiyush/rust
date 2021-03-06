// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

fn main() {
    test00();
    // test01();
    test02();
    test04();
    test05();
    test06();
}

fn test00_start(ch: ::core::oldcomm::Chan<int>, message: int, count: int) {
    debug!("Starting test00_start");
    let mut i: int = 0;
    while i < count {
        debug!("Sending Message");
        ::core::oldcomm::send(ch, message + 0);
        i = i + 1;
    }
    debug!("Ending test00_start");
}

fn test00() {
    let number_of_tasks: int = 1;
    let number_of_messages: int = 4;
    debug!("Creating tasks");

    let po = ::core::oldcomm::Port();
    let ch = ::core::oldcomm::Chan(&po);

    let mut i: int = 0;

    let mut results = ~[];
    while i < number_of_tasks {
        i = i + 1;
        do task::task().future_result(|+r| {
            results.push(move r);
        }).spawn |copy i| {
            test00_start(ch, i, number_of_messages);
        }
    }
    let mut sum: int = 0;
    for results.each |r| {
        i = 0;
        while i < number_of_messages { sum += ::core::oldcomm::recv(po); i = i + 1; }
    }

    for results.each |r| { r.recv(); }

    debug!("Completed: Final number is: ");
    assert (sum ==
                number_of_messages *
                    (number_of_tasks * number_of_tasks + number_of_tasks) /
                    2);
}

fn test01() {
    let p = ::core::oldcomm::Port();
    debug!("Reading from a port that is never written to.");
    let value: int = ::core::oldcomm::recv(p);
    log(debug, value);
}

fn test02() {
    let p = ::core::oldcomm::Port();
    let c = ::core::oldcomm::Chan(&p);
    debug!("Writing to a local task channel.");
    ::core::oldcomm::send(c, 42);
    debug!("Reading from a local task port.");
    let value: int = ::core::oldcomm::recv(p);
    log(debug, value);
}

fn test04_start() {
    debug!("Started task");
    let mut i: int = 1024 * 1024;
    while i > 0 { i = i - 1; }
    debug!("Finished task");
}

fn test04() {
    debug!("Spawning lots of tasks.");
    let mut i: int = 4;
    while i > 0 { i = i - 1; task::spawn(|| test04_start() ); }
    debug!("Finishing up.");
}

fn test05_start(ch: ::core::oldcomm::Chan<int>) {
    ::core::oldcomm::send(ch, 10);
    ::core::oldcomm::send(ch, 20);
    ::core::oldcomm::send(ch, 30);
    ::core::oldcomm::send(ch, 30);
    ::core::oldcomm::send(ch, 30);
}

fn test05() {
    let po = ::core::oldcomm::Port();
    let ch = ::core::oldcomm::Chan(&po);
    task::spawn(|| test05_start(ch) );
    let mut value: int;
    value = ::core::oldcomm::recv(po);
    value = ::core::oldcomm::recv(po);
    value = ::core::oldcomm::recv(po);
    log(debug, value);
}

fn test06_start(&&task_number: int) {
    debug!("Started task.");
    let mut i: int = 0;
    while i < 1000000 { i = i + 1; }
    debug!("Finished task.");
}

fn test06() {
    let number_of_tasks: int = 4;
    debug!("Creating tasks");

    let mut i: int = 0;

    let mut results = ~[];
    while i < number_of_tasks {
        i = i + 1;
        do task::task().future_result(|+r| {
            results.push(move r);
        }).spawn |copy i| {
            test06_start(i);
        };
    }


    for results.each |r| { r.recv(); }
}










