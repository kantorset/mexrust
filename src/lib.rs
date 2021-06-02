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
