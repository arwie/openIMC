use pyo3::prelude::*;


mod ethercat;
mod motion;


#[pyfunction]
fn move_joints(joints: Vec<f64>, speed:f64) -> PyResult<()> {
    Ok(motion::move_joints(joints, speed))
}

#[pyfunction]
fn get_joints() -> PyResult<Vec<f64>> {
    Ok(motion::get_joints())
}


#[pymodule]
fn openimc(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
	
	ethercat::init();
	motion::init();
	
    m.add_function(wrap_pyfunction!(move_joints, m)?)?;
    m.add_function(wrap_pyfunction!(get_joints, m)?)?;
    Ok(())
}
