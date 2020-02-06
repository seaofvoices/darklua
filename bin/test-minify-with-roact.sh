#!/bin/sh

# clone a known working version of roact
git clone https://github.com/Roblox/roact.git
cd roact
git checkout d9b7f9661b26ff16db240f2fe8b0f8284303c61d
git submodule init
git submodule update
cd ..

# minify the source
cargo run -- minify roact/src roact-minify/src

# copy the submodules and the bin to be able to execute tests
cp roact/bin roact-minify/bin -r
cp roact/modules roact-minify/modules -r

# run Lua tests
cd roact-minify
lua bin/spec.lua
