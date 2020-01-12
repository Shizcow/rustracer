use std::ops::{Mul, Add, Div};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64
}

impl Color {
    pub fn clamp(&mut self){
	if self.red < 0.0 {
	    self.red = 0.0;
	} else if self.red > 255.0 {
	    self.red = 255.0;
	}
	if self.green < 0.0 {
	    self.green = 0.0;
	} else if self.green > 255.0 {
	    self.green = 255.0;
	}
	if self.blue < 0.0 {
	    self.blue = 0.0;
	} else if self.blue > 255.0 {
	    self.blue = 255.0;
	}
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
	Color{red:   ((self.red   )*(other.red  )/255.0),
	      green: ((self.green )*(other.green)/255.0),
	      blue:  ((self.blue  )*(other.blue )/255.0)}
    }
}

