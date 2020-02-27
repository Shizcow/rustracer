#![allow(dead_code)]

extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate cgmath;

mod camera_math;
mod pixvec;
mod shapes;
mod color;
use crate::pixvec::*;
use crate::camera_math::*;
use crate::shapes::*;
use crate::color::*;
use crate::cgmath::InnerSpace;

use gio::prelude::*;
use gtk::prelude::*;
use std::env::args;
use gdk_pixbuf::Pixbuf;
use cgmath::Vector3;
use cgmath::Point3;

static WIDTH_RENDER    : usize = 2880;//1920;//640;
static HEIGHT_RENDER   : usize = 2160;//1440;//480;
static WIDTH_VIEWPORT  : i32   = 2880;
static HEIGHT_VIEWPORT : i32   = 2160;

fn render_scene(scene: &mut Scene, pvec: &mut Pixvec) {
    let focal_point = scene.camera.get_focal_point();
    for i in 0..pvec.height {
	for j in 0..pvec.width {
	    let p = scene.camera.pixel_to_world(j, i);
	    let ray = Ray{origin: p, direction: (p-focal_point).normalize()};
	    if let Some((color, white_balance)) = ray.trace(scene, 0) {
		pvec[i][j] = color;
		if white_balance > scene.white_balance {
		    scene.white_balance = white_balance;
		}
	    } // else - no collision
	}
    }
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Raytracer");
    window.set_position(gtk::WindowPosition::Center);

    let metal_texture = ImageMap::new_from_file("assets/metal.png".to_string(), 2.0);
    let static_texture = ImageMap::new_from_file("assets/static.jpg".to_string(), 5.0);

    let mut static_nodes = Vec::new();
    static_nodes.push(Node::Diffuse(ShadeDiffuse::new(1.0)));
    static_nodes.push(Node::Reflect(ShadeReflect::new(0.25)));
    let static_material = Material::new(Some(Texture::ImageMap(static_texture)), 0.3, static_nodes);
    
    let mut untextured_nodes = Vec::new();
    untextured_nodes.push(Node::Diffuse(ShadeDiffuse::new(1.0)));
    untextured_nodes.push(Node::Reflect(ShadeReflect::new(0.1)));
    let untextured_material = Material::new(None, 0.9, untextured_nodes);
    
    let mut chrome_nodes = Vec::new();
    chrome_nodes.push(Node::Diffuse(ShadeDiffuse::new(0.15)));
    chrome_nodes.push(Node::Reflect(ShadeReflect::new(1.0)));
    let chrome_material = Material::new(Some(Texture::Color(Color::new_from_linear(71, 221, 255))), 0.8, chrome_nodes);
    
    let mut blue_nodes = Vec::new();
    blue_nodes.push(Node::Refract(ShadeRefract::new(1.0, 1.5)));
    let blue_material = Material::new(Some(Texture::Color(Color::new_from_linear(100, 100, 255))), 1.0, blue_nodes);
    
    let mut backdrop_nodes = Vec::new();
    backdrop_nodes.push(Node::Diffuse(ShadeDiffuse::new(1.0)));
    let backdrop_material = Material::new(Some(Texture::Color(Color::new_from_linear(50, 50, 255))), 0.5, backdrop_nodes);
    
    let mut metal_nodes = Vec::new();
    metal_nodes.push(Node::Diffuse(ShadeDiffuse::new(1.0)));
    metal_nodes.push(Node::Reflect(ShadeReflect::new(1.0)));
    let metal_material = Material::new(Some(Texture::ImageMap(metal_texture)), 1.0, metal_nodes);
    
    let mut objects : Vec<SceneObject> = Vec::new();

    objects.push(SceneObject::Sphere(Sphere::new(Point3{x: 5.0, y:  -0.2, z: 1.3}, 0.3, chrome_material)));
    objects.push(SceneObject::Sphere(Sphere::new(Point3{x: 6.0, y:  -2.0, z: 3.0}, 0.3, static_material)));
    objects.push(SceneObject::Sphere(Sphere::new(Point3{x: 5.0, y: -0.5, z: 0.5}, 0.5, untextured_material)));
    objects.push(SceneObject::Sphere(Sphere::new(Point3{x: 4.5, y:  1.0, z: 1.5}, 1.0, blue_material)));
    objects.push(SceneObject::Plane(Plane::new(Point3{x: 0.0, y: 0.0, z: -0.3}, Vector3{x: 0.0, y: 0.0, z: -1.0}, metal_material)));
    objects.push(SceneObject::Plane(Plane::new(Point3{x: 100.0, y: 0.0, z: 0.0}, Vector3{x: 1.0, y: 0.0, z: 0.0}, backdrop_material)));
    
    for i in (-20..100).step_by(2) {
	let red_texture = ImageMap::new_from_file("assets/fire.jpg".to_string(), 5.0);
	let mut red_nodes = Vec::new();
	red_nodes.push(Node::Diffuse(ShadeDiffuse::new(1.0)));
	red_nodes.push(Node::Reflect(ShadeReflect::new(0.05)));
	let red_material = Material::new(Some(Texture::ImageMap(red_texture)), 0.9, red_nodes);
	objects.push(SceneObject::Sphere(Sphere::new(Point3{x: 1.0+i as f64, y: 1.0, z: 0.5}, 0.3, red_material)));
    }

    for i in (-20..100).step_by(2) {
	let red_texture = ImageMap::new_from_file("assets/fire.jpg".to_string(), 5.0);
	let mut red_nodes = Vec::new();
	red_nodes.push(Node::Diffuse(ShadeDiffuse::new(1.0)));
	red_nodes.push(Node::Reflect(ShadeReflect::new(0.05)));
	let red_material = Material::new(Some(Texture::ImageMap(red_texture)), 0.9, red_nodes);
	objects.push(SceneObject::Sphere(Sphere::new(Point3{x: 1.0+i as f64, y: -1.0, z: 0.5}, 0.3, red_material)));
    }
    /*
	let red_texture = ImageMap::new_from_file("assets/fire.jpg".to_string(), 5.0);
	let mut red_nodes = Vec::new();
	red_nodes.push(Node::Diffuse(ShadeDiffuse::new(1.0)));
	red_nodes.push(Node::Reflect(ShadeReflect::new(0.05)));
	let red_material = Material::new(Some(Texture::ImageMap(red_texture)), 0.9, red_nodes);
	objects.push(SceneObject::Sphere(Sphere::new(Point3{x: 3.0 as f64, y: -0.5, z: 0.5}, 0.3, red_material)));
     */
    
    let mut lights : Vec<SceneLight>  = Vec::new();
    lights.push(SceneLight::PointLight(PointLight::new(Point3{x: 60.0, y: 0.0, z: 150.0},
						       color::consts::WHITE,
						       1500000.0)));
    lights.push(SceneLight::PointLight(PointLight::new(Point3{x: -0.5, y: -3.0, z: 15.0},
						       color::consts::WHITE,
						       15000.0)));
    lights.push(SceneLight::PointLight(PointLight::new(Point3{x: 0.0, y: 3.0, z: 15.0},
						       color::consts::WHITE,
						       15000.0)));
    let mut scene = Scene{camera: Camera{location: Point3{x: 0.0, y: 0.0, z: -0.1},
					 rotation: Vector3{x: 0.0, y: 0.2, z: 0.0},
					 focal_length: 0.4,
					 resolution: Resolution{x: WIDTH_RENDER, y: HEIGHT_RENDER},
					 hx: 0.5,
					 hy: 0.375},
			  objects: objects,
			  lights: lights,
			  white_balance: 0.0};
    
    let mut pvec = Pixvec::new(WIDTH_RENDER, HEIGHT_RENDER);
    render_scene(&mut scene, &mut pvec);
    let mut pbuf = Pixbuf::from(&mut pvec).scale_simple(WIDTH_VIEWPORT, HEIGHT_VIEWPORT, gdk_pixbuf::InterpType::Bilinear).unwrap();
    if let Err(error) = pbuf.savev("out.png", "png", &[]) {
	println!("Could not save image! {:?}", error);
    }
    let image = gtk::Image::new_from_pixbuf(Some(&mut pbuf));
    let event_box = gtk::EventBox::new();

    event_box.add(&image);

    window.add(&event_box);
    window.show_all();

}

fn main() {
    let application = gtk::Application::new(
        Some("com.shizcow.rustracer"),
        Default::default(),
    )
	.expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
