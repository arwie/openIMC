#include "rust/cxx.h"
#include <memory>
#include <ruckig/ruckig.hpp>

using namespace std;
using namespace ruckig;


Ruckig<5> otg {0.004};
InputParameter<5> input;
OutputParameter<5> output;
Result status;

const double vmax = 200;


void init() {
	const double amax = vmax*1000/1200;		input.max_acceleration	= { amax, amax, amax, amax, amax };
	const double jmax = amax*1000/600;		input.max_jerk			= { jmax, jmax, jmax, jmax, jmax };
	//input.synchronization = Synchronization::Phase;
}


void move_joints(array<double, 5> joints, double speed) {
	input.target_position = joints;
	double v = vmax * speed;
	input.max_velocity = { v, v, v, v, v };
}


array<double, 5> update() {
	status = otg.update(input, output);

	output.pass_to_input(input);
	
	array<double, 5> joints;
	copy(output.new_position.begin(), output.new_position.begin() + 5, joints.begin());
	return joints;
}
