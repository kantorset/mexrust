MEX_FUNCTION_NAME=mex_rust
MEX_EXTENSION=mex #change to mexa64 for matlab on linux (mexw64 for matlab on windows, etc)


mex_debug: export MEX_LIB_NAME=octave #change this to mex for matlab
mex_debug: export MEX_LIB_PATH=/usr/lib/x86_64-linux-gnu/ #For matlab change this to path to matlab directory with libmex.so e.g. on linux /usr/local/MATLAB/R<matlab version>/bin/glnxa64
mex_debug: src/lib.rs
	cargo build
	cp -p target/debug/libmexrust.so ${MEX_FUNCTION_NAME}.${MEX_EXTENSION}

mex_release: export MEX_LIB_NAME=octave #change this to mex for matlab
mex_release: export MEX_LIB_PATH=/usr/lib/x86_64-linux-gnu/ #For matlab change this to path to matlab directory with libmex.so e.g. on linux /usr/local/MATLAB/R<matlab version>/bin/glnxa64
mex_release: src/lib.rs
	cargo build --release
	cp -p target/release/libmexrust.so ${MEX_FUNCTION_NAME}.${MEX_EXTENSION}

