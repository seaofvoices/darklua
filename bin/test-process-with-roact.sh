#!/bin/sh

# clone a known working version of roact
git clone https://github.com/Roblox/roact.git
cd roact
git checkout d9b7f9661b26ff16db240f2fe8b0f8284303c61d
git submodule init
git submodule update
cd ..

# process the source
cargo run -- process roact/src roact-process/src

# copy the submodules and the bin to be able to execute tests
cp roact/bin roact-process/bin -r
cp roact/modules roact-process/modules -r

# run Lua tests
cd roact-process
lua bin/spec.lua

cd ..
rm -r -f roact
rm -r -f roact-process
