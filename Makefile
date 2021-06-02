MEX_FUNCTION_NAME=mex_rust
MEX_EXTENSION=mex #change to mexa64 for matlab on linux (mexw64 for matlab on windows, etc)


mex_debug: export MEX_LIB_NAME=octave #change this to mex for matlab
mex_debug: export MEX_LIB_PATH=/usr/lib/x86_64-linux-gnu/ #change to path to matlab lib directory with libmex.so
mex_debug: src/lib.rs
	cargo build
	cp -p target/debug/libmexrust.so ${MEX_FUNCTION_NAME}.${MEX_EXTENSION}

mex_release: export MEX_LIB_NAME=octave #change this to mex for matlab
mex_release: export MEX_LIB_PATH=/usr/lib/x86_64-linux-gnu/ #change to path to matlab lib directory with libmex.so
mex_release: src/lib.rs
	cargo build --release
	cp -p target/release/libmexrust.so ${MEX_FUNCTION_NAME}.${MEX_EXTENSION}

