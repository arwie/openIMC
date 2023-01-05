use std::{thread, time, sync::Mutex};
use thread_priority::*;

mod ruckig;


#[derive(Debug)]
struct Target {
	joints: [f64;5],
	speed: f64,
}

struct RtData {
	joints_cmd: [f64;5],
	target: Option<Target>,
}


static RT_DATA: Mutex<RtData> = Mutex::new(RtData {joints_cmd:[0.0;5], target:None});


fn rt() -> () {
	println!("rt thread started");

	ruckig::ffi::init();
	
	set_thread_priority_and_policy(
		thread_native_id(),
		ThreadPriority::from_posix(ScheduleParams {sched_priority: 50}),
		ThreadSchedulePolicy::Realtime(RealtimeThreadSchedulePolicy::Fifo)
	).unwrap();
	
	let mut jnt = [0.0;5];
	loop {
		
		{
			let rt_data = &mut RT_DATA.lock().unwrap();
			rt_data.joints_cmd = jnt;
			if let Some(target) = rt_data.target.take() {
				ruckig::ffi::move_joints(target.joints, target.speed);
			}
		}
		
		jnt = ruckig::ffi::update();
		
		thread::sleep(time::Duration::from_millis(10));
	}
}


pub fn init() -> () {
	println!("motion init");
	thread::spawn(rt);
}


pub fn move_joints(jnt: Vec<f64>, speed:f64) -> () {
	let target = Target {joints:[jnt[0],jnt[1],jnt[2],jnt[3],jnt[4]], speed};
	println!("moveJoints: {:?}", &target);
	RT_DATA.lock().unwrap().target = Some(target);
}

pub fn get_joints() -> Vec<f64> {
	RT_DATA.lock().unwrap().joints_cmd.to_vec()
}
