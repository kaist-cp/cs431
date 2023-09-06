#!/usr/bin/env bash

mkdir -p target
zip target/hw-hello_server.zip -j src/hello_server/cache.rs src/hello_server/tcp.rs src/hello_server/thread_pool.rs
zip target/hw-list_set.zip -j src/list_set/fine_grained.rs src/list_set/optimistic_fine_grained.rs
zip target/hw-hash_table.zip -j src/hash_table/growable_array.rs src/hash_table/split_ordered_list.rs
zip target/hw-hazard_pointer.zip -j src/hazard_pointer/hazard.rs src/hazard_pointer/retire.rs
