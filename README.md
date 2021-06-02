# MexRust

MexRust is a proof of concept library demonstrating the generation of Mex functions (for interfacing with MATLAB / Octave) from rust. This was developed by extending https://github.com/nclack/mexrs which provided a very minimal example but did not have functionality to actually pass data from Matlab or Octave into rust. This was developed partly as a project for a relatively inexperienced rust user to experiment with rust and the rust FFI so caveat emptor. 
  

## Usage


### lib.rs
MEX functions are basically just plugins for MATLAB. More precisely, they are just shared object libraries with a special entry point for MATLAB/Octave named ```mexFunction``` any shared object library linked against ```libmex``` or ```liboctave``` with a ```mexFunction``` symbol and the correct file extension can be called as a mex function. For this rust/mex example we just create a rust dylib and put the rust code we want to call from MATLAB/Octave in an ```extern "C"``` function called ```mexFunction```. The unsafe code interacting with the MATLAB API is in the ```mex_rust``` module and called through the ```MexInterface``` struct. Here is an example mexFunction that does nothing particularly useful other than demonstrating passing data back and forth between rust and matlab with rather pointless computations done in rust.

```rust
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use std::os::raw::c_int;

use rayon::prelude::*; //This is not necessary for the matlab / rust interface but used as an 
                       //example of parallel computation on data received from matlab / octave


mod mex_rust;
pub use mex_rust::mex_rust::*;


#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn mexFunction(nlhs: c_int,
                              plhs: *mut *mut mxArray,
                              nrhs: c_int,
                              prhs: *mut *mut mxArray) {

    let mx = MexInterface::new(nlhs,plhs,nrhs,prhs); // package up the input (prhs,nrhs) and output (nlhs,plhs) mxArray pointers

    let input_0 = mx.get_real_as_slice::<f64>(0).unwrap(); // wrapped_input is Option<WrappedMex<'a,T> >, 0 indicates the first input
    let input_1 = mx.get_real_as_slice::<f32>(1).unwrap(); //get_real gets real (no imaginary part) and is generic over the type, 1 indicates the second input
    let scalar_input = mx.get_real_scalar::<i64>(2); //We can also get scalars (here an int64)
    let big_array = mx.get_real_as_slice::<f32>(3).unwrap();
    let input_string = mx.get_string(4).unwrap(); //And receive strings

    display_to_mex("You passed in: ".to_owned()+&input_string); //display_to_mex will print a displayable object back to the matlab /octave console

    //Use rayon to do a parallel sum over columns

    let parallel_mapped:Vec<f32> = (0..big_array.N).into_par_iter().map(|i| {
                let a = &big_array.data[(i*big_array.M)..(i+1)*big_array.M];
                a.iter().fold(0.0,|sum,val| sum+val)
            }
          ).collect();

    let output_0 = mx.create_real_as_slice::<f64>(0,input_0.M,input_0.N); //Create a matlab / octave array to be returned in th 0th slot
                                                                          // Same array shape as wrapped_input (input_0.M x input_0.N)
    
    //multiply element-wise the first two arrays we received
    for (i,x) in output_0.unwrap().data.iter_mut().enumerate(){
        *x = input_0.data[i]*(input_1.data[i] as f64);
    }

    //Create an array of int32 to be returned in slot 1 and fill it with 0..24
    let output_1 = mx.create_real_as_slice::<i32>(1,5,5);

    for (i,x) in output_1.unwrap().data.iter_mut().enumerate(){
        *x=i as i32;
    }

    //do a fold and display the result
    display_to_mex(input_0.data.iter().fold(0.0,|a,b| a+b));

    //Return a scalar in slot 2
    mx.return_real_scalar(2,scalar_input.unwrap()*2);

    //Return the output of the rayon computation (Here a copy of the vector is performed) in slot 3
    mx.return_real_vector(3,1,parallel_mapped.len(),parallel_mapped);

}
```
### Makefile
A simple makefile is provided 
```makefile
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
```

To build the mex run ```make mex_debug``` or ```make mex_release```.  It will build it and then copy the resulting shared object library to the top-level directory with whatever name is specified. The project is configured by default to compile for octave. To compile against matlab you need to change the ```MEX_LIB_NAME``` to ```mex``` and give the path to the directory containing ```libmex``` in your matlab install. The name of the resulting mex can be configured with the ```MEX_FUNCTION_NAME``` variable. The extension may need to be changed depending on the platform (linux/windows matlab/octave). 

### Usage

After compiling the above example we can use it as follows.

```matlab
>> m=single(randn(1000,1000));
>> [a,b,c,d]=mex_rust([10.0,3.0,5.0,2.0],single([5.0,6.0,7.0,8.0]),int64(10),m,'test');
You passed in: test
20
>> size(d)
ans =

      1   1000

>> sum(d-sum(m,1))
ans = 0
>>
```

### mex_rust.rs
The underlying code to do the interfacing with the matlab mex api is in ```mex_rust.rs``` . It defines a wrapper class to wrap a raw matlab pointer in a slice. 

```rust
    pub struct WrappedMex<'a,T>{
        pub data: &'a mut [T],
        pub M: usize,
        pub N: usize
    }
```


It also defines the MexInterface class which packages the pointers to the inputs and outputs received from Matlab/Octave and provides methods to extract them into rust and pass back to matlab.
```rust
    pub struct MexInterface{
        nlhs: c_int,
        plhs: *mut *mut mxArray,
        nrhs: c_int,
        prhs: *mut *mut mxArray
    }
```

## Limitations / TODO

There are many ways this could be improved / extended
  1. Currently complex (real + imaginary) arrays are not supported. This is made annoyingly messy by the fact that octave's mex interface exposes separate real and imaginary pointers for complex data while MATLAB 2018b and later uses interleaved real / imaginary data but prior to 2018b it also used a split real and imaginary representation.  
  2. Currently only 2D arrays are supported, support for higher dimensional arrays would be straightforward. 
  3. Structs and Struct arrays are not supported. Macros could probably be used to enable passing (compatible) rust structs to MATLAB / Octave and receiving MATLAB / Octave structs into rust struct. This would likely require some Macros to derive some MEXSerializable attribute or something. An analagous capability is present in https://github.com/kantorset/MexPackUnpack but using C++ template metaprogamming.
  4. Cell arrays are not supported. 

## License
[MIT](LICENSE.txt)