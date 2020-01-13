use std::ops::{Mul, Add, Div, AddAssign};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Color {
    red: f64,
    green: f64,
    blue: f64
}

impl Color {
    pub fn clamp(&mut self){
	if self.red < 0.0 {
	    self.red = 0.0;
	} else if self.red > 1.0 {
	    self.red = 1.0;
	}
	if self.green < 0.0 {
	    self.green = 0.0;
	} else if self.green > 1.0 {
	    self.green = 1.0;
	}
	if self.blue < 0.0 {
	    self.blue = 0.0;
	} else if self.blue > 1.0 {
	    self.blue = 1.0;
	}
    }
    fn p_srgb_to_linear(pixel: u8) -> f64 {
	let x = (pixel as f64) / 255.0;
	let y = 
	    if x <= 0.0 {
		0.0
	    } else if x >= 1.0 {
		1.0
	    } else if x < 0.04045 {
		x / 12.92
	    } else {
		((x + 0.055) / 1.055).powf(2.4)
	    };
	y
    }
    fn p_linear_to_srgb(pixel: f64) -> u8 {
	let x = pixel; // 0.0 <= x <= 1.0
	let y = 
	if x <= 0.0 {
	    0.0
	} else if x >= 1.0 {
	    1.0
	} else if x < 0.0031308 {
	    x * 12.92
	} else {
	    x.powf(1.0 / 2.4) * 1.055 - 0.055
	};
	(y*255.0) as u8
    }
    pub fn linear_to_srgb(&mut self) -> (u8, u8, u8) {
	(Color::p_linear_to_srgb(self.red),
	 Color::p_linear_to_srgb(self.green),
	 Color::p_linear_to_srgb(self.blue))
    }
    pub fn new_from_srgb(red: u8, green: u8, blue: u8) -> Self {
	Color{red: Color::p_srgb_to_linear(red),
	      green: Color::p_srgb_to_linear(green),
	      blue: Color::p_srgb_to_linear(blue)}
    }
    pub fn new_from_linear(red: u8, green: u8, blue: u8) -> Self {
	Color{red: red as f64, blue: blue as f64, green: green as f64}/255.0
    }
}

impl AddAssign<Color> for Color {
    fn add_assign(&mut self, other: Color) {
	self.red += other.red; 
	self.green += other.green; 
	self.blue += other.blue; 
    }
}

impl Add<Color> for Color {
    type Output = Color;
    fn add(self, other: Color) -> Color {
	Color{red:   self.red+other.red,
	      green: self.green+other.green,
	      blue:  self.blue+other.blue}
    }
}

impl Div<f64> for Color {
    type Output = Color;
    fn div(self, other: f64) -> Color {
	self * ( 1.0 / other )
    }
}

impl Mul<f64> for Color {
    type Output = Color;
    fn mul(self, other: f64) -> Color {
	Color{red:   self.red  *other,
	      green: self.green*other,
	      blue:  self.blue *other}
    }
}

impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, other: Color) -> Color {
	Color{red:   self.red  *other.red  ,
	      green: self.green*other.green,
	      blue:  self.blue *other.blue }
    }
}

