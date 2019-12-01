#!/usr/bin/env bash

rm -rf homework
cp -a ../cs492-concur-homework homework
rm -rf homework/.git

mv homework/src/art/mod_skeleton.rs homework/src/art/mod.rs
mv homework/src/bst/mod_skeleton.rs homework/src/bst/mod.rs
mv homework/src/elim_stack/elim_skeleton.rs homework/src/elim_stack/elim.rs
