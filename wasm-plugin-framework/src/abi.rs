use bincode::{
    config::{Bounded, WithOtherLimit},
    DefaultOptions, Options,
};

lazy_static::lazy_static! {
    static ref BINCODE_OPTIONS: WithOtherLimit<DefaultOptions, Bounded> = DefaultOptions::new().with_limit(u32::MAX as u64);
}

fn from_bytes<T>(bytes: &[u8]) -> T
where
    T: for<'a> serde::Deserialize<'a>,
{
    BINCODE_OPTIONS
        .deserialize(bytes)
        .expect("Unexpect error decoding bincode-encoded ABI message")
}

fn into_bytes<T>(v: &T) -> Vec<u8>
where
    T: serde::Serialize + ?Sized,
{
    BINCODE_OPTIONS
        .serialize(&v)
        .expect("Error serialising bincode-encoded value")
}

#[cfg(target_arch="wasm32")]
pub mod wasm32 {
    pub fn from_abi<T>(ptr: u32) -> T
    where
        T: for<'a> serde::Deserialize<'a>,
    {
        let size_ptr = ptr as *mut u32;

        // ! ###  Unsafe conditions not to be violated

        // * size_ptr must be valid for reads.
        // * size_ptr must point to a properly initialized value of type T.
        let size = u32::from_le(unsafe { size_ptr.read_unaligned() }) as usize;

        // ! ###  Unsafe conditions not to be violated

        // * ptr needs to have been previously allocated via String/Vec<T> (at least, it's highly likely to be incorrect if it wasn't).
        // * T needs to have the same size and alignment as what ptr was allocated with. (T having a less strict alignment is not sufficient, the alignment really needs to be equal to satisfy the dealloc requirement that memory must be allocated and deallocated with the same layout.)
        // * length needs to be less than or equal to capacity.
        // * capacity needs to be the capacity that the pointer was allocated with.
        let s = unsafe { Vec::from_raw_parts(ptr as *mut u8, size + 4, size + 4) };
        super::from_bytes(&s[4..])
    }

	pub fn into_abi<T>(t: &T) -> u32
    where
        T: serde::Serialize,
    {
        let mut s = super::into_bytes(&t);
		let mut v = s.len().to_le_bytes().to_vec();
		v.append(&mut s);
		v = v.into_boxed_slice().into_vec();
		let ptr = v.as_mut_ptr() as u32;
		std::mem::forget(v);
		ptr
    }

	#[no_mangle]
	/// Allocates a full, empty vec, with the same capacity as the size provided (IT DOES NOT ADD 4 BYTES TO THE SIZE)
	pub extern "C" fn allocate_buffer(size: u32) -> u32 {
		let mut v: Vec<u8> = Vec::with_capacity(size as usize);
		let ptr = v.as_mut_ptr() as u32;
		std::mem::forget(v);
		ptr
	}

	#[no_mangle]
	/// Frees a buffer of `size` bytes, allocated with this API (IT DOES NOT ADD 4 BYTES TO THE SIZE)
	/// It frees it by loading it as a Vec and then dropping it
	pub extern "C" fn free_buffer(ptr: u32, size: u32) {
		// ! ###  Unsafe conditions not to be violated

        // * ptr needs to have been previously allocated via String/Vec<T> (at least, it's highly likely to be incorrect if it wasn't).
        // * T needs to have the same size and alignment as what ptr was allocated with. (T having a less strict alignment is not sufficient, the alignment really needs to be equal to satisfy the dealloc requirement that memory must be allocated and deallocated with the same layout.)
        // * length needs to be less than or equal to capacity.
        // * capacity needs to be the capacity that the pointer was allocated with.
		unsafe { Vec::from_raw_parts(ptr as *mut u8, size as usize, size as usize) };
	}
}
#[cfg(target_arch="wasm32")]
pub use wasm32::*;

#[cfg(not(target_arch="wasm32"))]
pub mod not_wasm32 {
    use serde::Serialize;
    use wasmer::Memory;

	pub trait PluginLoader {
		fn allocate_buffer(&self, size: u32) -> u32;
		fn free_buffer(&self, ptr: u32, size: u32);
		fn memory(&self) -> &Memory;
	}

	pub fn into_abi<P, T>(plugin_loader: &P, t: &T) -> u32 where T: Serialize, P: PluginLoader {
		let v = super::into_bytes(t);
		let ptr = plugin_loader.allocate_buffer(v.len() as u32 + 4) as usize;
		let m = plugin_loader.memory();
		{
			// ! ###  Unsafe conditions not to be violated

			//* This method provides interior mutability without an UnsafeCell. Until the returned value is dropped,
			//* it is undefined behaviour to read or write to the pointed-to memory in any way except through this slice,
			//* including by calling a wasm function that reads the memory contents or by resizing this Memory.
			let slice_mut = unsafe {m.data_unchecked_mut()};
			slice_mut[ptr..ptr+4].copy_from_slice(&(v.len() as u32).to_le_bytes());
			slice_mut[ptr+4..ptr+4+v.len()].copy_from_slice(&v);
		};
		ptr as u32
	}

}
#[cfg(not(target_arch="wasm32"))]
pub use not_wasm32::*;
