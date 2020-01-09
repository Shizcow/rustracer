use cgmath::Vector3;
use cgmath::Point3;

pub struct Resolution {
    pub x: usize,
    pub y: usize
}

pub struct Camera {
    pub location: Point3<f64>,
    pub rotation: Vector3<f64>,
    pub focal_length: f64,
    pub resolution: Resolution,
    pub hx: f64,
    pub hy: f64
}

impl Camera {
    pub fn get_focal_point(&self) -> Point3<f64> { // point E
	let z_offset = self.focal_length*self.rotation.y.sin();
	let f_sh = self.focal_length*self.rotation.y.cos();
	let x_offset = f_sh*self.rotation.z.cos();
	let y_offset = f_sh*self.rotation.z.sin();
	self.location-Vector3{x: x_offset, y: y_offset, z:z_offset}
    }
    pub fn pixel_to_world(&self, x: usize, y: usize) -> Point3<f64> {
	// first, calculate the point when rotations all equal 0
	let x_offset = self.hx*((x as f64 + 0.5)/(self.resolution.x as f64)-0.5);
	let y_offset = self.hy*((0.5 - y as f64)/(self.resolution.y as f64)+0.5); // y=0 is top of image so need to treat different
	let mut rotate = Vector3{x: 0.0, y: x_offset, z: y_offset};
	// then, rotate the point vector accordingly
	{ // first, along x
	    let (tmp_y, tmp_z) = (rotate.y, rotate.z);
	    rotate.y = tmp_y*self.rotation.x.cos()-tmp_z*self.rotation.x.sin();
	    rotate.z = tmp_z*self.rotation.x.cos()+tmp_y*self.rotation.x.sin();
	}
	{ // second, along y
	    let (tmp_x, tmp_z) = (rotate.x, rotate.z);
	    rotate.z = tmp_z*self.rotation.y.cos()-tmp_x*self.rotation.y.sin();
	    rotate.x = tmp_z*self.rotation.y.sin()+tmp_x*self.rotation.y.cos();
	}
	{ // third, along z
	    let (tmp_x, tmp_y) = (rotate.x, rotate.y);
	    rotate.x = tmp_x*self.rotation.z.cos()-tmp_y*self.rotation.z.sin();
	    rotate.y = tmp_x*self.rotation.z.sin()+tmp_y*self.rotation.z.cos();
	}
	self.location+rotate // finally, relative to location
    }
}
