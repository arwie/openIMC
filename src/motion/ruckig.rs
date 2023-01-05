
#[cxx::bridge]
pub mod ffi {

	unsafe extern "C++" {
		include!("openimc/src/motion/ruckig.cc");

		fn init() -> ();
		fn move_joints(joints:[f64;5], speed:f64) -> ();
		fn update() -> [f64;5];

	}

}
