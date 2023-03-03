use std::{thread, time, sync::Mutex};
use thread_priority::*;
use std::time::{Duration, Instant};
use std::cmp::max;

mod ruckig;


#[derive(Debug)]
struct Target {
	joints: [f64;5],
	speed: f64,
}

struct RtData {
	joints_cmd: [f64;5],
	target: Option<Target>,
	jitter: Duration,
	calc: Duration,
}


static RT_DATA: Mutex<RtData> = Mutex::new(RtData {joints_cmd:[0.0;5], target:None, jitter:Duration::ZERO, calc:Duration::ZERO});


fn rt() -> () {
	println!("rt thread started");

	ruckig::ffi::init();
	
	set_thread_priority_and_policy(
		thread_native_id(),
		ThreadPriority::from_posix(ScheduleParams {sched_priority: 40}),
		ThreadSchedulePolicy::Realtime(RealtimeThreadSchedulePolicy::Fifo)
	).unwrap();
	
	let mut jnt = [0.0;5];
	
	let mut cycle = Instant::now();
	loop {
		let wakeup = Instant::now();
		
		{
			let rt_data = &mut RT_DATA.lock().unwrap();
			rt_data.jitter = max(rt_data.jitter, wakeup - cycle);
			rt_data.joints_cmd = jnt;
			
			if let Some(target) = rt_data.target.take() {
				ruckig::ffi::move_joints(target.joints, target.speed);
			}
			
			jnt = ruckig::ffi::update();
			crate::ethercat::process(jnt[2]);
			
			rt_data.calc = max(rt_data.calc, Instant::now() - wakeup);
		}
		
		cycle += Duration::from_micros(4000);
		thread::sleep(cycle - Instant::now());
	}
}


pub fn init() -> () {
	println!("motion init");
	thread::spawn(rt);
}


pub fn move_joints(jnt: Vec<f64>, speed:f64) -> () {
	let target = Target {joints:[jnt[0],jnt[1],jnt[2],jnt[3],jnt[4]], speed};
	println!("moveJoints: {:?}", &target);
	let rt_data = &mut RT_DATA.lock().unwrap();
	rt_data.target = Some(target);
	println!("jitter:{}µs  calc:{}µs", rt_data.jitter.as_micros(),  rt_data.calc.as_micros());
	//rt_data.jitter = Duration::ZERO;
	//rt_data.calc = Duration::ZERO;
}

pub fn get_joints() -> Vec<f64> {
	RT_DATA.lock().unwrap().joints_cmd.to_vec()
}
