#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
pub mod mex_rust{

    use std::os::raw::c_int;
    use std::os::raw::c_char;
    use std::slice;
    use std::ffi::CString;
    use std::ffi::CStr;
    use core::fmt::Display;

    //use std::marker::PhantomData;

    // required mex types

    pub type mxArray = u64; //Fake mxArray type, type is irrelevant only interact with pointer to mxArray in mex functions


    //mex types, from octave header, probably the same as matlab, need to check
    #[repr(C)]
    pub enum mxClassID{
      mxUNKNOWN_CLASS,
      mxCELL_CLASS,
      mxSTRUCT_CLASS,
      mxLOGICAL_CLASS,
      mxCHAR_CLASS,
      mxVOID_CLASS,
      mxDOUBLE_CLASS,
      mxSINGLE_CLASS,
      mxINT8_CLASS,
      mxUINT8_CLASS,
      mxINT16_CLASS,
      mxUINT16_CLASS,
      mxINT32_CLASS,
      mxUINT32_CLASS,
      mxINT64_CLASS,
      mxUINT64_CLASS,
      mxFUNCTION_CLASS
    }



    pub struct MexInterface{
        nlhs: c_int,
        plhs: *mut *mut mxArray,
        nrhs: c_int,
        prhs: *mut *mut mxArray
    }





    //from octave header, probably the same for matlab need to check
    #[repr(C)]
    pub enum mxClassType{
        mxREAL,
        mxCOMPLEX
    }


    //matlab / octave C API functions 
    //#[link(name="octave")]
    extern "C" {
        fn mexPrintf(fmt: *const u8, ...);
        fn mxGetPr(mx: *mut mxArray)-> *mut f64;
        fn mxArrayToString(mx: *mut mxArray)-> *mut c_char;
        fn mxGetM(mx: *mut mxArray)-> usize;
        fn mxGetN(mx: *mut mxArray)-> usize;
        fn mxCreateNumericMatrix(m: usize, n: usize,classtype: mxClassID,complexity: mxClassType)-> *mut mxArray;
    }



    /*
        Struct representing a wrapped matlab/octave array. The field data is a slice referencing the underlying data of the matlab/octave 2D array with M and N its size. 
    */
    pub struct WrappedMex<'a,T>{
        pub data: &'a mut [T],
        pub M: usize,
        pub N: usize
    }


    unsafe impl<'a,T> Send for WrappedMex<'a,T>{}

    // Trait signifying which scalar types have a corresponding matlab/octave array type
    pub trait MexScalarClass{
        fn getClass()->mxClassID;
    }


    impl MexScalarClass for f32 {
        fn getClass()->mxClassID{
            return mxClassID::mxSINGLE_CLASS;
        }
    }

    impl MexScalarClass for f64 {
        fn getClass()->mxClassID{
            return mxClassID::mxDOUBLE_CLASS;
        }
    }

    impl MexScalarClass for i32 {
        fn getClass()->mxClassID{
            return mxClassID::mxINT32_CLASS;
        }
    }

    impl MexScalarClass for i64 {
        fn getClass()->mxClassID{
            return mxClassID::mxINT64_CLASS;
        }
    }


    impl MexScalarClass for u32 {
        fn getClass()->mxClassID{
            return mxClassID::mxUINT32_CLASS;
        }
    }

    impl MexScalarClass for u64 {
        fn getClass()->mxClassID{
            return mxClassID::mxUINT64_CLASS;
        }
    }


    impl MexScalarClass for u16 {
        fn getClass()->mxClassID{
            return mxClassID::mxUINT16_CLASS;
        }
    }

    impl MexScalarClass for u8 {
        fn getClass()->mxClassID{
            return mxClassID::mxUINT8_CLASS;
        }
    }


    impl MexScalarClass for i16 {
        fn getClass()->mxClassID{
            return mxClassID::mxINT16_CLASS;
        }
    }

    impl MexScalarClass for i8 {
        fn getClass()->mxClassID{
            return mxClassID::mxINT8_CLASS;
        }
    }



    impl MexInterface{
        pub fn new(nlhs: c_int,plhs: *mut *mut mxArray,nrhs: c_int, prhs: *mut *mut mxArray)->Self{
            MexInterface{nlhs:nlhs,plhs:plhs,nrhs:nrhs,prhs:prhs}
        }



        pub fn get_string (&self,argnum: i32)->Option<String>
        {
            if argnum<self.nrhs {
                unsafe{
                    let raw_pointer: *mut c_char = mxArrayToString(*self.prhs.offset(argnum as isize));
                    let rust_string = CStr::from_ptr(raw_pointer).to_str();
                    match rust_string {
                        Ok(s)=>Some(s.to_string()),
                        Err(_e)=>None
                    }
                }
            } else{
                return None;
            }
        }


    
    // Function which extracts a matlab/octave array passed into the mex function from slot argnum. 
    // The underlying pointer is assumed to correspond to data of type T. Results are wrapped in a WrappedMex structure
        pub fn get_real_as_slice<'a,T> (&self,argnum: i32)->Option<WrappedMex<'a,T> >
        {
            if argnum<self.nrhs {
                unsafe{
                    let raw_pointer: *mut T = mxGetPr(*self.prhs.offset(argnum as isize)) as *mut T;
                    let M: usize = mxGetM(*self.prhs.offset(argnum as isize));
                    let N: usize = mxGetN(*self.prhs.offset(argnum as isize ));        
                    return Some(WrappedMex {data: slice::from_raw_parts_mut(raw_pointer,M*N),M:M,N:N});
                }
            } else{
                return None;
            }
        }

        // Function which extracts scalar passed into the mex function from slot argnum. 
        // The underlying pointer is assumed to correspond to data of type T. 
        pub fn get_real_scalar<T:Copy> (&self,argnum: i32)->Option<T>
        {
            if argnum<self.nrhs {
                unsafe{
                    let raw_pointer: *mut T = mxGetPr(*self.prhs.offset(argnum as isize)) as *mut T;
                    return Some(*raw_pointer);
                }
            } else{
                return None;
            }
        }


        // Function which creates a matlab/octave array to be returned to matlab/octave. 
        // Pointer of underlying array is assigned to slot argnum on the left hand side.
        // The data is returned to rust as a WrappedMex object.
        pub fn create_real_as_slice<'a,T: MexScalarClass> (&self,argnum: i32, M: usize,N: usize)->Option<WrappedMex<'a,T> >
        {
            if argnum<self.nlhs {
                unsafe{
                    let out_array = mxCreateNumericMatrix(M,N,T::getClass(),mxClassType::mxREAL);            
                    *self.plhs.offset(argnum as isize) = out_array;
                    let raw_pointer: *mut T = mxGetPr( out_array) as *mut T;
                    return Some(WrappedMex {data: slice::from_raw_parts_mut(raw_pointer,M*N),M:M,N:N});
                }
            } else{
                return None;
            }
        }


        pub fn return_real_scalar<T:Copy+MexScalarClass> (&self,argnum: i32, output:T)
        {
            if argnum<self.nlhs {
                unsafe{
                    let out_array = mxCreateNumericMatrix(1,1,T::getClass(),mxClassType::mxREAL);            
                    *self.plhs.offset(argnum as isize) = out_array;
                    let raw_pointer: *mut T = mxGetPr( out_array) as *mut T;
                    *raw_pointer=output;
                }
            } 
        }


        
        pub fn return_real_vector<T:Copy+MexScalarClass> (&self,argnum: i32, M:usize, N:usize,v: Vec<T>)
        {
            if argnum<self.nlhs {
                unsafe{
                    let out_array = mxCreateNumericMatrix(M,N,T::getClass(),mxClassType::mxREAL);            
                    *self.plhs.offset(argnum as isize) = out_array;
                    let raw_pointer: *mut T = mxGetPr( out_array) as *mut T;
                    v.iter().enumerate().for_each(|(i,x)| *raw_pointer.offset(i as isize)=*x);
//                    *raw_pointer=output;
                }
            } 
        }
        
    }
    
    pub fn display_to_mex<T:Display>(val: T){
        unsafe {
            let c_str = CString::new(val.to_string()+"\n").unwrap();
            mexPrintf(c_str.as_ptr() as *const u8);
        }
    }

}


    /*
    pub fn createDoubleMexAsSlice<'a> (plhs:*mut *mut mxArray,nlhs: i32,argnum: i32, M: usize,N: usize)->Option<wrappedMex<'a,f64> >
    {
        if argnum<nlhs {
            unsafe{
                let out_array = mxCreateNumericMatrix(M,N,mxClassID::mxDOUBLE_CLASS,mxClassType::mxREAL);            
                *plhs.offset(argnum as isize) = out_array;
                let raw_pointer: *mut f64 = mxGetPr( out_array);
                return Some(wrappedMex {data: slice::from_raw_parts_mut(raw_pointer,M*N),M:M,N:N});
            }
        } else{
            return None;
        }
    }
    */
//pub unsafe fn pointer_to_slice<'a, T> (data: *mut T, total_size: usize)->&'a [T] {
//    slice::from_raw_parts(data,total_size)
//}
