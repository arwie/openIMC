fn main() {
	cxx_build::bridge("src/motion/ruckig.rs")
		.include("ruckig/include")
		.file("src/motion/ruckig.cc")
		.flag_if_supported("-std=c++20")
		.compile("openimc");
		
		println!("cargo:rustc-link-lib=ruckig")
}
