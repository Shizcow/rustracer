extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate cgmath;

mod camera_math;
mod pixvec;
mod shapes;
mod commontypes;
use crate::pixvec::*;
use crate::camera_math::*;
use crate::shapes::*;
use crate::commontypes::*;
use crate::cgmath::InnerSpace;

use gio::prelude::*;
use gtk::prelude::*;
use std::env::args;
use gdk_pixbuf::Pixbuf;
use cgmath::Vector3;
use cgmath::Point3;

static WIDTH_RENDER    : usize = 1920;//640;
static HEIGHT_RENDER   : usize = 1440;//480;
static WIDTH_VIEWPORT  : i32   = 1920;
static HEIGHT_VIEWPORT : i32   = 1440;

fn render_scene(scene: &mut Scene, pvec: &mut Pixvec) {
    let focal_point = scene.camera.get_focal_point();
    for i in 0..pvec.height {
	for j in 0..pvec.width {
	    let p = scene.camera.pixel_to_world(j, i);
	    let ray = Ray{origin: p, direction: (p-focal_point).normalize()};
	    if let Some((color, white_balance)) = ray.prime_bounce(scene) {
		pvec[i][j] = color;
		if white_balance > scene.white_balance {
		    scene.white_balance = white_balance;
		}
	    } // else - no collision
	}
    }
    for i in 0..pvec.height {
	for j in 0..pvec.width {
	    pvec[i][j] = pvec[i][j] * (1.0 / scene.white_balance);
	}
    }
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Raytracer");
    window.set_position(gtk::WindowPosition::Center);

    let metal_texture = ImageMap::new_from_file("assets/metal.png".to_string(), 2.0);
    let static_texture = ImageMap::new_from_file("assets/static.jpg".to_string(), 5.0);
    let mut objects : Vec<SceneObject> = Vec::new();

    objects.push(SceneObject::Sphere(Sphere::new(Point3{x: 5.0, y:  0.0, z: 0.0}, 0.3, Material{texture: Some(Texture::ImageMap(static_texture)), albedo: 0.9})));
    objects.push(SceneObject::Sphere(Sphere::new(Point3{x: 5.0, y: -0.5, z: 0.5}, 0.5, Material{texture: None, albedo: 0.9})));
    objects.push(SceneObject::Sphere(Sphere::new(Point3{x: 4.5, y:  0.7, z: 0.7}, 0.7, Material{texture: Some(Texture::Color(Color{red: 0.0,   green: 0.0,   blue: 255.0})), albedo: 0.9})));
    objects.push(SceneObject::Plane(Plane::new(Point3{x: 0.0, y: 0.0, z: -0.5}, Vector3{x: 0.0, y: 0.0, z: -1.0}, Material{texture: Some(Texture::ImageMap(metal_texture)), albedo: 1.0})));
    
    let mut lights  : Vec<SceneLight>  = Vec::new();
    lights.push(SceneLight::Sun(Sun::new(Vector3{x: 0.3, y: 0.2, z: -0.8},
				    Color{red: 255.0, green: 255.0, blue: 255.0},
				    0.25)));
    lights.push(SceneLight::Sun(Sun::new(Vector3{x: 0.1, y: -0.2, z: -0.8},
				    Color{red: 255.0, green: 255.0, blue: 255.0},
					 0.15)));
    /*
    lights.push(SceneLight::PointLight(PointLight::new(Point3{x: 0.0, y: 0.0, z: 15.0},
						       Color{red: 255.0, green: 250.0, blue: 250.0},
						       30000.0)));
    lights.push(SceneLight::PointLight(PointLight::new(Point3{x: 20.0, y: -4.0, z: 0.5},
						       Color{red: 255.0, green: 250.0, blue: 250.0},
						       45.0)));
    lights.push(SceneLight::PointLight(PointLight::new(Point3{x: 30.0, y: -6.0, z: 0.5},
						       Color{red: 255.0, green: 250.0, blue: 250.0},
						       45.0)));
    lights.push(SceneLight::PointLight(PointLight::new(Point3{x: 40.0, y: -8.0, z: 0.5},
						       Color{red: 255.0, green: 250.0, blue: 250.0},
						       45.0)));
    lights.push(SceneLight::PointLight(PointLight::new(Point3{x: 4.0, y: 0.3, z: 0.15},
						       Color{red: 255.0, green: 150.0, blue: 150.0},
						       5.0)));
    lights.push(SceneLight::PointLight(PointLight::new(Point3{x: 4.4, y: -0.35, z: 0.15},
						       Color{red: 255.0, green: 150.0, blue: 150.0},
						       5.0)));*/
    let mut scene = Scene{camera: Camera{location: Point3{x: 0.0, y: 0.0, z: 0.5},
					 rotation: Vector3{x: 0.0, y: -0.06, z: 0.0},
					 focal_length: 1.0,
					 resolution: Resolution{x: WIDTH_RENDER, y: HEIGHT_RENDER},
					 hx: 0.5,
					 hy: 0.375},
			  objects: objects,
			  lights: lights,
			  white_balance: 0.0};
    
    let mut pvec = Pixvec::new(WIDTH_RENDER, HEIGHT_RENDER);
    render_scene(&mut scene, &mut pvec);
    let mut pbuf = Pixbuf::from(&mut pvec).scale_simple(WIDTH_VIEWPORT, HEIGHT_VIEWPORT, gdk_pixbuf::InterpType::Nearest).unwrap();
    let image = gtk::Image::new_from_pixbuf(Some(&mut pbuf));
    let event_box = gtk::EventBox::new();

    event_box.add(&image);

    window.add(&event_box);
    window.show_all();

}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.treeview"),
        Default::default(),
    )
	.expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
