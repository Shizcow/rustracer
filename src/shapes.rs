use crate::commontypes::*;
use crate::camera_math::Camera;
use crate::cgmath::InnerSpace;
use crate::cgmath::MetricSpace;
use cgmath::Point3;
use cgmath::Vector3;

#[allow(dead_code)]
pub enum Texture {
    Color(Color)
}

#[allow(dead_code)]
pub struct Material {
    pub texture: Option<Texture>,
    pub albedo: f64
}

#[allow(dead_code)]
pub struct Sphere {
    origin: Point3<f64>,
    radius: f64,
    pub material: Material
}

#[allow(dead_code)]
impl Sphere {
    pub fn new(
	origin: Point3<f64>,
	radius: f64,
	material: Material) -> Self {
	Self{origin: origin, radius: radius, material: material}
    }                                  // distance --v  v--location  v--direction  v-- surface normal
    pub fn intersects(&self, ray: &Ray) -> Option<(f64, Point3<f64>, Vector3<f64>, Vector3<f64>)> {
	let ray_to_sphere = self.origin - ray.origin;
	let adj = ray_to_sphere.dot(ray.direction);
	let frinde_radius2 = ray_to_sphere.dot(ray_to_sphere) - (adj * adj); //squared
	let radius2 = self.radius * self.radius; // squared
	if frinde_radius2 > radius2 {
            return None; // miss
	}
	let half_chord = (radius2 - frinde_radius2).sqrt();
	let intersect_front = adj - half_chord;
	let intersect_back = adj + half_chord; // check for ray inside sphere

	if intersect_front < 0.0 && intersect_back < 0.0 {
            return None; // wrong direction
	}

	let distance = intersect_front.min(intersect_back);
	let intersection_point = ray.origin+(ray.direction*distance);
	let surface_normal = (intersection_point-self.origin).normalize();
	let reflection_vector = ray.direction-2.0*ray.direction.dot(surface_normal)*surface_normal;
	
	Some((distance, intersection_point, reflection_vector.normalize(), surface_normal))
    }
    fn get_texture_coords(&self, location: &Point3<f64>) -> (f64, f64) {
	let v = location-self.origin;
	let phi = (v[1]/v[0]).atan();
	let theta = (v[2]/self.radius).acos();
	(if phi >= 0.0 {phi} else {2.0*std::f64::consts::PI+phi},if theta >= 0.0 {theta} else {2.0*std::f64::consts::PI+theta})
    }
    
    pub fn get_texture_color(&self, location: &Point3<f64>) -> Color {
	match &self.material.texture {
	    Some(Texture::Color(color)) => *color,
	    None => {
		let (mut phi, theta) = self.get_texture_coords(location);
		phi += std::f64::consts::PI;
		if ((phi/(std::f64::consts::PI))%0.5 < 0.25) ^ ((theta/(std::f64::consts::PI))%0.5 < 0.25) {
		    Color{red: 225.0, green: 255.0, blue: 225.0}
		} else {
		    Color{red: 0.0, green: 0.0, blue: 0.0}
		}
	    }
	}
    }
}

#[allow(dead_code)]
pub struct Plane {
    pub origin: Point3<f64>,
    pub normal: Vector3<f64>,
    pub material: Material
}


#[allow(dead_code)]
impl Plane {
    pub fn new(origin: Point3<f64>,
	       normal: Vector3<f64>,
	       material: Material) -> Self {
	Self{origin: origin, normal: normal.normalize(), material: material}
    }                                  // distance --v  v--location  v--direction  v-- surface normal
    pub fn intersects(&self, ray: &Ray) -> Option<(f64, Point3<f64>, Vector3<f64>, Vector3<f64>)> {
        let proj = self.normal.dot(ray.direction);
        if proj > 0.0 { // anything less than parallel
            let distance = (self.origin - ray.origin).dot(self.normal) / proj;
            if distance >= 0.0 { // in direction of ray
		// ray.direction is already normalized, so scaling & adding it will result in:
		let intersection_point = ray.origin+(ray.direction*distance);
		// finally, the reflection is found as:
		let surface_normal = -self.normal;
		let reflection_vector = ray.direction-2.0*ray.direction.dot(surface_normal)*surface_normal;
                return Some((distance, intersection_point, reflection_vector.normalize(), surface_normal));
            } else {
		None
	    }
        } else {
            None
	}
    }
    fn get_texture_coords(&self, location: &Point3<f64>) -> (f64, f64) {
	let v = location-self.origin;
	let mut x_axis = self.normal.cross(Vector3{
	    x: 0.0,
	    y: 0.0,
	    z: 1.0,
	});
	if x_axis.magnitude() == 0.0 {
	    x_axis = self.normal.cross(Vector3{
		x: 0.0,
		y: 1.0,
		z: 0.0,
	    });
	}
	let y_axis = self.normal.cross(x_axis);
	(v.dot(x_axis), v.dot(y_axis))
    }
    
    pub fn get_texture_color(&self, location: &Point3<f64>) -> Color {
	match &self.material.texture {
	    Some(Texture::Color(color)) => *color,
	    None => {
		let (mut x, mut y) = self.get_texture_coords(location);
		if x < 0.0 {
		    x = 0.25-x;
		}
		if y < 0.0 {
		    y = 0.25-y;
		}
		if (x%0.5 < 0.25) ^ (y%0.5 < 0.25) {
		    Color{red: 225.0, green: 255.0, blue: 225.0}
		} else {
		    Color{red: 0.0, green: 0.0, blue: 0.0}
		}
	    }
	}
    }
}

#[allow(dead_code)]
pub struct Sun {
    pub direction: Vector3<f64>,
    pub color: Color,
    pub intensity: f64
}

#[allow(dead_code)]
impl Sun {
    pub fn new(direction: Vector3<f64>,
	       color: Color,
	       intensity: f64) -> Self {
	Sun{direction: direction.normalize(), color: color, intensity: intensity}
    }
}

#[allow(dead_code)]
pub struct PointLight {
    pub origin: Point3<f64>,
    pub color: Color,
    pub intensity: f64
}

#[allow(dead_code)]
impl PointLight {
    pub fn new(position: Point3<f64>,
	       color: Color,
	       intensity: f64) -> Self {
	PointLight{origin: position, color: color, intensity: intensity}
    }
}


#[allow(dead_code)]
pub enum SceneLight {
    Sun(Sun),
    PointLight(PointLight)
}

impl SceneLight {
    pub fn dist_to(&self, point: Point3<f64>) -> f64 {
	match *self {
            SceneLight::Sun(ref _s) => std::f64::MAX,
            SceneLight::PointLight(ref p) => {
		p.origin.distance(point)
	    }
        }
    }
    pub fn get_apparent_intensity(&self, dp: f64, distance: f64) -> f64 {
	match *self {
            SceneLight::Sun(ref s) => (
		if dp < 0.0 {0.0}
		else {dp*s.intensity}
	    ),
            SceneLight::PointLight(ref p) => {
		if dp < 0.0 {0.0}
		else {dp*p.intensity/(4.0*std::f64::consts::PI*distance.powi(2))}
	    },
        }
    }
    pub fn get_direction(&self, point: Point3<f64>) -> Vector3<f64> {
	match *self {
            SceneLight::Sun(ref s) => -s.direction,
            SceneLight::PointLight(ref p) => {
		(p.origin-point).normalize()
	    }
        }
    }
    pub fn get_color(&self) -> &Color {
	match *self {
            SceneLight::Sun(ref s) => &s.color,
            SceneLight::PointLight(ref p) => &p.color
        }
    }
}

#[allow(dead_code)]
pub enum SceneObject {
    Sphere(Sphere),
    Plane(Plane)
}

#[allow(dead_code)]
impl SceneObject {
    pub fn get_albedo(&self) -> f64 {
	match *self {
            SceneObject::Sphere(ref s) => s.material.albedo,
            SceneObject::Plane(ref p) => p.material.albedo,
        }
    }
    pub fn get_texture_color(&self, location: &Point3<f64>) -> Color {
	match *self {
            SceneObject::Sphere(ref s) => s.get_texture_color(location),
            SceneObject::Plane(ref p) => p.get_texture_color(location),
        }
    }
    pub fn intersects(&self, ray: &Ray) -> Option<(f64, Point3<f64>, Vector3<f64>, Vector3<f64>)> {
        match *self {
            SceneObject::Sphere(ref s) => s.intersects(ray),
            SceneObject::Plane(ref p) => p.intersects(ray),
        }
    }
}

pub struct Ray {
    pub origin: Point3<f64>,
    pub direction: Vector3<f64>
}

impl Ray {
    fn any_intersect(&self, scene: &Scene, target_distance: f64) -> bool {
	//simply checks if there's an intersection before a target distance
	let mut hit_something = false;
	
	for scene_object in scene.objects.iter() {
	    if let Some((dist, _location, _direction, _normal)) = scene_object.intersects(self) {
		if dist <= target_distance {
		    hit_something = true;
		    break;
		}
	    }
	}
	hit_something
    }
    fn closest_intersect<'a>(&self, scene: &'a Scene) -> Option<(f64, Point3<f64>, Vector3<f64>, Vector3<f64>, &'a SceneObject)> {
	// finds the closest intersection and returns an Option with the following in order:
	// distance, location of intersection, reflection direction, surface normal of object at reflected point, reference to object (for color, etc.)
	let mut intersection : Option<(f64, Point3<f64>, Vector3<f64>, Vector3<f64>, &'a SceneObject)> = None;
	for scene_object in scene.objects.iter() {
	    if let Some((dist, location, reflection, normal)) = scene_object.intersects(self) {
		if !intersection.is_some() || intersection.unwrap().0 > dist {
		    intersection = Some((dist, location, reflection, normal, scene_object));
		}
	    }
	}
	intersection
    } //                                                            v-- power @ pixel
    pub fn prime_bounce(&self, scene: &Scene) -> Option<(Color, f64)> { // from direction of next
	let mut white_balance = 0.0;
	let ret = 
	    if let Some((_dist, location, _reflection_vector, surface_normal, obj)) = self.closest_intersect(scene) {
		let mut mix = Vector3::<f64>::new(0.0, 0.0, 0.0);
		for light in scene.lights.iter() {
		    let new_origin = location+surface_normal*1e-13;
		    let dir_to_light = light.get_direction(new_origin);
		    let dist_to_light = light.dist_to(new_origin);
		    let shadow_ray = Ray{origin: new_origin, direction: dir_to_light};

		    if !shadow_ray.any_intersect(scene, dist_to_light) {
			let dp = surface_normal.dot(dir_to_light);
			let power =  light.get_apparent_intensity(dp, dist_to_light)
			    * obj.get_albedo();
			white_balance += power;

			let color = (obj.get_texture_color(&location))
			    * (*light.get_color()) * power;
			
			mix[0] += color.red   as f64;
			mix[1] += color.green as f64;
			mix[2] += color.blue  as f64;
		    }
		}
		mix = mix/(scene.lights.len() as f64); // take average
		Some((Color{red: mix[0], green: mix[1], blue: mix[2]}, white_balance/(scene.lights.len() as f64)))
	    } else {
		None
	    };
	ret
    }
}


pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<SceneObject>,
    pub lights: Vec<SceneLight>,
    pub white_balance: f64
}
