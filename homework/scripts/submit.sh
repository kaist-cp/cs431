#!/usr/bin/env bash

mkdir -p target
zip target/hw1.zip -j src/hello_server/cache.rs src/hello_server/tcp.rs src/hello_server/thread_pool.rs
zip target/hw5.zip -j src/hash_table/growable_array.rs src/hash_table/split_ordered_list.rs
zip target/hw6.zip -j src/hazard_pointer/hazard.rs src/hazard_pointer/mod.rs src/hazard_pointer/retire.rs
