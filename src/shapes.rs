use crate::commontypes::*;
use crate::pixvec::*;
use crate::camera_math::Camera;
use crate::cgmath::InnerSpace;
use crate::cgmath::MetricSpace;
use cgmath::Point3;
use cgmath::Vector3;

static RECURSION_DEPTH : i32 = 35;
static NORMAL_BIAS     : f64 = 1e-13; // used for shadow ache and such

pub struct ImageMap {
    pub pixvec: Pixvec,
    pub scale: f64
}


impl ImageMap {
    pub fn new_from_file(uri: String, scale: f64) -> Self {
	ImageMap{pixvec: Pixvec::from(&gdk_pixbuf::Pixbuf::new_from_file(uri).unwrap()), scale: scale}
    }
}


pub enum Texture {
    Color(Color),
    ImageMap(ImageMap)
}

pub struct ShadeDiffuse {
    strength: f64
}

impl ShadeDiffuse{
    pub fn shade_diffuse(&self, scene: &Scene, location: Point3<f64>, surface_normal: Vector3<f64>, obj: &SceneObject) -> Color {
	let mut mix = Vector3::<f64>::new(0.0, 0.0, 0.0);
	let new_origin = location+surface_normal*NORMAL_BIAS;
	for light in scene.lights.iter() {
	    let dir_to_light = light.get_direction(new_origin);
	    let dist_to_light = light.dist_to(new_origin);
	    let shadow_ray = Ray{origin: new_origin, direction: dir_to_light};

	    if !shadow_ray.any_intersect(scene, dist_to_light) {
		let dp = surface_normal.dot(dir_to_light);
		let power =  light.get_apparent_intensity(dp, dist_to_light)
		    * obj.get_albedo();

		let color = (obj.get_texture_color(&new_origin))
		    * (*light.get_color()) * power;
		
		mix[0] += color.red   as f64;
		mix[1] += color.green as f64;
		mix[2] += color.blue  as f64;
	    }
	}
	mix = mix/(scene.lights.len() as f64); // take average
	Color{red: mix[0], green: mix[1], blue: mix[2]}*self.strength
    }
    pub fn new(strength: f64) -> Self {
	ShadeDiffuse{strength: strength}
    }
}

pub struct ShadeReflect {
    strength: f64
}

impl ShadeReflect {
    pub fn shade_reflect(&self, scene: &Scene, location: Point3<f64>, incident: Vector3<f64>, surface_normal: Vector3<f64>, _obj: &SceneObject, n_th: i32) -> Color {
	if n_th < RECURSION_DEPTH {
	    let reflection_vector = (incident-2.0*incident.dot(surface_normal)*surface_normal).normalize();
	    let reflection_ray = Ray{origin: location+surface_normal*NORMAL_BIAS, direction: reflection_vector};
	    if let Some((color, _power)) = reflection_ray.trace(scene, n_th+1) {
		color*self.strength
	    } else {
		Color{red: 0.0, green: 0.0, blue: 0.0}
	    }
	} else {
	    Color{red: 0.0, green: 0.0, blue: 0.0}
	}
    }
    pub fn new(strength: f64) -> Self {
	ShadeReflect{strength: strength}
    }
}

pub struct ShadeRefract {
    strength: f64,
    index: f64
}

impl ShadeRefract {
    pub fn shade_refract(&self, scene: &Scene, location: Point3<f64>, incident: Vector3<f64>, surface_normal: Vector3<f64>, obj: &SceneObject, n_th: i32) -> Color {
	if n_th < RECURSION_DEPTH {
	    let dp = incident.dot(surface_normal);
	    let ref_dp = if dp < 0.0 {-dp} else {dp}; // correct based on inside or outside
	    let ref_n = if dp < 0.0 {surface_normal} else {-surface_normal};
	    
	    let eta = if dp < 0.0 {1.0/self.index} else {self.index}; // set up total refractive index
	    
	    let dist2 = 1.0 - (eta * eta) * (1.0 - ref_dp * ref_dp); // direction of refraction
	    if dist2 < 0.0 { // wrong direction -- ignore
		Color{red: 0.0, green: 0.0, blue: 0.0}
	    } else {
		if let Some((color, _power)) = (Ray{ // trace refraction
		    origin: location - ref_n*NORMAL_BIAS,
		    direction: (incident + ref_dp*ref_n)*eta - ref_n*dist2.sqrt(),
		}).trace(scene, n_th+1) {
		    color*self.strength*obj.get_texture_color(&location)
		} else {
		    Color{red: 0.0, green: 0.0, blue: 0.0} // no collision after refraction
		}
	    }
	} else {
	    Color{red: 0.0, green: 0.0, blue: 0.0} // overflow
	}
    }
    pub fn new(strength: f64, index: f64) -> Self {
	ShadeRefract{strength: strength, index: index}
    }
}

pub struct ShadeFresnel {
    strength: f64,
    index: f64
}

impl ShadeFresnel {
    pub fn new(strength: f64, index: f64) -> Self {
	ShadeFresnel{strength: strength, index: index}
    }
    fn fresnel(&self, incident: Vector3<f64>, normal: Vector3<f64>, index: f64) -> f64 {
	/*
	let sin_t = eta_i / eta_t * (1.0 - dp * dp).max(0.0).sqrt();
	if sin_t > 1.0 {
            1.0
	} else {
            let cos_t = (1.0 - sin_t * sin_t).max(0.0).sqrt();
            let cos_i = cos_t.abs();
            let r_s = ((eta_t * cos_i) - (eta_i * cos_t)) / ((eta_t * cos_i) + (eta_i * cos_t));
            let r_p = ((eta_i * cos_i) - (eta_t * cos_t)) / ((eta_i * cos_i) + (eta_t * cos_t));
            (r_s * r_s + r_p * r_p) / 2.0
	}
	 */
	let i_dot_n = incident.dot(normal);
	let mut eta_i = 1.0;
	let mut eta_t = index as f64;
	if i_dot_n > 0.0 {
            eta_i = eta_t;
            eta_t = 1.0;
	}

	let sin_t = eta_i / eta_t * (1.0 - i_dot_n * i_dot_n).max(0.0).sqrt();
	if sin_t > 1.0 {
            //Total internal reflection
            return 1.0;
	} else {
            let cos_t = (1.0 - sin_t * sin_t).max(0.0).sqrt();
            let cos_i = cos_t.abs();
            let r_s = ((eta_t * cos_i) - (eta_i * cos_t)) / ((eta_t * cos_i) + (eta_i * cos_t));
            let r_p = ((eta_i * cos_i) - (eta_t * cos_t)) / ((eta_i * cos_i) + (eta_t * cos_t));
            return (r_s * r_s + r_p * r_p) / 2.0;
	}
    }
    pub fn shade_fresnel(&self, scene: &Scene, location: Point3<f64>, incident: Vector3<f64>, surface_normal: Vector3<f64>, obj: &SceneObject, n_th: i32) -> Color {
	let mut refraction_color = Color{red: 0.0, green: 0.0, blue: 0.0};
        let kr = self.fresnel(incident, surface_normal, self.index);

        if kr < 1.0 {
            refraction_color = (ShadeRefract{strength: 1.0, index: self.index}).shade_refract(scene, location, incident, surface_normal, obj, n_th+1);
        }

        let reflection_color =  (ShadeReflect{strength: 1.0}).shade_reflect(scene, location, incident, surface_normal, obj, n_th+1);
        let mut color = reflection_color * kr + refraction_color * (1.0 - kr);
        color = color * self.strength;
        color
    }
}

pub enum Node {
    Diffuse(ShadeDiffuse),
    Reflect(ShadeReflect),
    Refract(ShadeRefract),
    Fresnel(ShadeFresnel)
}

impl Node {
    pub fn resolve(&self, scene: &Scene, location: Point3<f64>, surface_normal: Vector3<f64>, incident: Vector3<f64>, obj: &SceneObject, n_th: i32) -> Color {
	match *self {
            Node::Diffuse(ref n) => n.shade_diffuse(scene, location, surface_normal, obj),
            Node::Reflect(ref n) => n.shade_reflect(scene, location, incident, surface_normal, obj, n_th),
            Node::Refract(ref n) => n.shade_refract(scene, location, incident, surface_normal, obj, n_th),
            Node::Fresnel(ref n) => n.shade_fresnel(scene, location, incident, surface_normal, obj, n_th),
        }
    }
    pub fn get_strength(&self) -> f64 {
	match *self {
            Node::Diffuse(ref n) => n.strength,
            Node::Reflect(ref n) => n.strength,
            Node::Refract(ref n) => n.strength,
            Node::Fresnel(ref n) => n.strength
        }
    }
    pub fn set_strength(&mut self, new_strength: f64){
	match *self {
            Node::Diffuse(ref mut n) => n.strength = new_strength,
            Node::Reflect(ref mut n) => n.strength = new_strength,
            Node::Refract(ref mut n) => n.strength = new_strength,
            Node::Fresnel(ref mut n) => n.strength = new_strength
        }
    }
}


pub struct Material {
    pub texture: Option<Texture>,
    pub albedo: f64,
    pub nodes: Vec<Node>
}


impl Material {
    pub fn new(texture: Option<Texture>, albedo: f64, mut nodelist: Vec<Node>) -> Self {
	let mut magnitude: f64 = 0.0; // normalize nodes
	for node in nodelist.iter() {
	    magnitude += node.get_strength().powi(2);
	}
	magnitude = magnitude.sqrt();
	for node in nodelist.iter_mut() {
	    node.set_strength(node.get_strength()/magnitude);
	}
	Material{texture: texture, albedo: albedo, nodes: nodelist}
    }
}

pub struct Sphere {
    origin: Point3<f64>,
    radius: f64,
    pub material: Material
}


impl Sphere {
    pub fn new(
	origin: Point3<f64>,
	radius: f64,
	material: Material) -> Self {
	Self{origin: origin, radius: radius, material: material}
    }                                  // distance --v  v--location  v-- surface normal
    pub fn intersects(&self, ray: &Ray) -> Option<(f64, Point3<f64>, Vector3<f64>)> {
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

	let distance = 
	    if intersect_front < 0.0 {
		intersect_back
	    } else if intersect_back < 0.0 {
		intersect_front
	    } else {
		intersect_front.min(intersect_back)
	    };
	let intersection_point = ray.origin+(ray.direction*distance);
	let surface_normal = (intersection_point-self.origin).normalize();
	
	Some((distance, intersection_point, surface_normal))
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
	    Some(Texture::ImageMap(ImageMap{pixvec, scale})) => {
		let (mut x, mut y) = self.get_texture_coords(location);
		x /= std::f64::consts::PI;
		y /= std::f64::consts::PI;
		if pixvec.width > pixvec.height {
		    x %= scale;
		    y %= scale*(pixvec.height as f64)/(pixvec.width as f64);
		    if x < 0.0 {
			x += scale;
		    }
		    if y < 0.0 {
			y += scale*(pixvec.height as f64)/(pixvec.width as f64);
		    }
		    pixvec[(y*(pixvec.height as f64)/(scale*(pixvec.height as f64)/(pixvec.width as f64))) as usize][(x*(pixvec.width as f64)/scale) as usize]
		} else {
		    x %= scale*(pixvec.width as f64)/(pixvec.height as f64);
		    y %= scale;
		    if x < 0.0 {
			x += scale*(pixvec.width as f64)/(pixvec.height as f64);
		    }
		    if y < 0.0 {
			y += scale;
		    }
		    pixvec[(y*(pixvec.height as f64)/scale) as usize][(x*(pixvec.width as f64)/(scale*(pixvec.width as f64)/(pixvec.height as f64))) as usize]
		}
	    },
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


pub struct Plane {
    pub origin: Point3<f64>,
    pub normal: Vector3<f64>,
    pub material: Material
}



impl Plane {
    pub fn new(origin: Point3<f64>,
	       normal: Vector3<f64>,
	       material: Material) -> Self {
	Self{origin: origin, normal: normal.normalize(), material: material}
    }                                  // distance --v  v--location  v-- surface normal
    pub fn intersects(&self, ray: &Ray) -> Option<(f64, Point3<f64>, Vector3<f64>)> {
        let proj = self.normal.dot(ray.direction);
        if proj > 0.0 { // anything less than parallel
            let distance = (self.origin - ray.origin).dot(self.normal) / proj;
            if distance >= 0.0 { // in direction of ray
		// ray.direction is already normalized, so scaling & adding it will result in:
		let intersection_point = ray.origin+(ray.direction*distance);
		// finally, the reflection is found as:
		let surface_normal = -self.normal;
                return Some((distance, intersection_point, surface_normal));
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
	    Some(Texture::ImageMap(ImageMap{pixvec, scale})) => {
		let (mut x, mut y) = self.get_texture_coords(location);
		if pixvec.width > pixvec.height {
		    x %= scale;
		    y %= scale*(pixvec.height as f64)/(pixvec.width as f64);
		    if x < 0.0 {
			x += scale;
		    }
		    if y < 0.0 {
			y += scale*(pixvec.height as f64)/(pixvec.width as f64);
		    }
		    pixvec[(y*(pixvec.height as f64)/(scale*(pixvec.height as f64)/(pixvec.width as f64))) as usize][(x*(pixvec.width as f64)/scale) as usize]
		} else {
		    x %= scale*(pixvec.width as f64)/(pixvec.height as f64);
		    y %= scale;
		    if x < 0.0 {
			x += scale*(pixvec.width as f64)/(pixvec.height as f64);
		    }
		    if y < 0.0 {
			y += scale;
		    }
		    pixvec[(y*(pixvec.height as f64)/scale) as usize][(x*(pixvec.width as f64)/(scale*(pixvec.width as f64)/(pixvec.height as f64))) as usize]
		}
	    },
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


pub struct Sun {
    pub direction: Vector3<f64>,
    pub color: Color,
    pub intensity: f64
}


impl Sun {
    pub fn new(direction: Vector3<f64>,
	       color: Color,
	       intensity: f64) -> Self {
	Sun{direction: direction.normalize(), color: color, intensity: intensity}
    }
}


pub struct PointLight {
    pub origin: Point3<f64>,
    pub color: Color,
    pub intensity: f64
}


impl PointLight {
    pub fn new(position: Point3<f64>,
	       color: Color,
	       intensity: f64) -> Self {
	PointLight{origin: position, color: color, intensity: intensity}
    }
}



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


pub enum SceneObject {
    Sphere(Sphere),
    Plane(Plane)
}


impl SceneObject {
    pub fn get_nodes(&self) -> &Vec<Node> {
	match *self {
            SceneObject::Sphere(ref s) => &s.material.nodes,
            SceneObject::Plane(ref p) => &p.material.nodes,
        }
    }
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
    pub fn intersects(&self, ray: &Ray) -> Option<(f64, Point3<f64>, Vector3<f64>)> {
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
	    if let Some((dist, _location, _normal)) = scene_object.intersects(self) {
		if dist <= target_distance {
		    hit_something = true;
		    break;
		}
	    }
	}
	hit_something
    }
    fn closest_intersect<'a>(&self, scene: &'a Scene) -> Option<(f64, Point3<f64>, Vector3<f64>, &'a SceneObject)> {
	// finds the closest intersection and returns an Option with the following in order:
	// distance, location of intersection, surface normal of object at reflected point, reference to object (for color, etc.)
	let mut intersection : Option<(f64, Point3<f64>, Vector3<f64>, &'a SceneObject)> = None;
	for scene_object in scene.objects.iter() {
	    if let Some((dist, location, normal)) = scene_object.intersects(self) {
		if !intersection.is_some() || intersection.unwrap().0 > dist {
		    intersection = Some((dist, location, normal, scene_object));
		}
	    }
	}
	intersection
    } //                                                        v-- power @ pixel
    pub fn trace(&self, scene: &Scene, n_th: i32) -> Option<(Color, f64)> { // from direction of next
	let ret = 
	    if let Some((_dist, location, surface_normal, obj)) = self.closest_intersect(scene) {
		let mut color_tally = Color{red: 0.0, green: 0.0, blue: 0.0};
		for node in obj.get_nodes() {
		    color_tally = color_tally + node.resolve(scene, location, surface_normal, self.direction, obj, n_th+1);
		}
		Some((color_tally, 0.0))
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
