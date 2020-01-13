use std::ops::{Index, IndexMut};
use crate::commontypes::*;

#[allow(dead_code)]
pub struct Pixvec { // two dimensional
    data: Vec<Vec<Color>>,
    pub colorspace: gdk_pixbuf::Colorspace,
    pub has_alpha: bool,
    pub bits_per_sample: i32,
    pub width: usize,
    pub height: usize,
    pub rowstride: usize
}

#[allow(dead_code)]
impl Pixvec {
    pub fn new(width: usize, // create new, assume RGB, no alpha
	       height: usize) -> Pixvec {
	let mut data = Vec::with_capacity(height as usize);
	data.resize_with(height as usize, || vec![Color::new_from_linear(0, 0, 0); width as usize]);
	Pixvec {data: data, colorspace: gdk_pixbuf::Colorspace::Rgb, has_alpha: false, bits_per_sample: 8, width: width, height: height, rowstride: 3*width}
    }
    pub fn new_from_vec(mut data: Vec<Vec<Color>>,
			colorspace: gdk_pixbuf::Colorspace,
			has_alpha: bool,
			bits_per_sample: i32,
			width: usize,
			height: usize,
			rowstride: usize) -> Pixvec {
	for row in data.iter_mut() {
	    if row.len() < width as usize {
		row.resize_with(width as usize, || Color::new_from_linear(0, 0, 0));
	    }
	}
	if data.len() < height as usize {
	    data.resize_with(height as usize, || vec![Color::new_from_linear(0, 0, 0); width as usize]);
	}
	Pixvec {data: data, colorspace: colorspace, has_alpha: has_alpha, bits_per_sample: bits_per_sample, width: width, height: height, rowstride: rowstride}
    }
    pub fn copy(&self) -> Pixvec {
	Pixvec::new_from_vec(self.data.to_vec(), self.colorspace, self.has_alpha, self.bits_per_sample, self.width, self.height, self.rowstride)
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Vec<Color>> {
	self.data.iter()
    }
    pub fn into_iter(self) -> std::vec::IntoIter<Vec<Color>> {
	self.data.into_iter()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Vec<Color>> {
	self.data.iter_mut()
    }
}

// casting to/from Pixbuf
impl From<&gdk_pixbuf::Pixbuf> for Pixvec {
    fn from(pbuf: &gdk_pixbuf::Pixbuf) -> Self {
	if pbuf.get_has_alpha() {
	    panic!("Pixelbuffer has Alpha, can't continue");
	}
	if pbuf.get_colorspace() != gdk_pixbuf::Colorspace::Rgb {
	    panic!("Pixelbuffer isn't in Rgb, can't continue");
	} // assuming images come in as sRGB and need to be converted to linear
	let bytes = pbuf.read_pixel_bytes().unwrap();
	let height = pbuf.get_height() as usize;
	let rowstride = pbuf.get_rowstride() as usize;
	let mut data : Vec<Vec<Color>> = Vec::with_capacity(height);
	let width = pbuf.get_width() as usize;
	let padding = rowstride%width;
	for i in 0..height {
	    let mut row : Vec<Color> = Vec::with_capacity(width);
	    for j in (3*i*width+i*padding..3*(i+1)*width+(i+1)*padding-padding).step_by(3) {
		row.push(Color::new_from_srgb(*bytes.get(j).unwrap(), *bytes.get(j+1).unwrap(), *bytes.get(j+2).unwrap()));
	    }
	    data.push(row);
	}
	Pixvec::new_from_vec(data,
		    pbuf.get_colorspace(),
		    pbuf.get_has_alpha(),
		    pbuf.get_bits_per_sample(), 
		    width,
		    height,
		    rowstride-padding)
    }
}

impl From<&mut Pixvec> for gdk_pixbuf::Pixbuf {
    fn from(pvec: &mut Pixvec) -> Self {
	let mut bytes : Vec<u8> = Vec::with_capacity(pvec.height*pvec.width*3);
	for row in pvec.iter_mut() {
	    for rgb in row.iter_mut() {
		rgb.clamp();
		let gamma_corrected_rgb = rgb.linear_to_srgb();
		bytes.push(gamma_corrected_rgb.0);
		bytes.push(gamma_corrected_rgb.1);
		bytes.push(gamma_corrected_rgb.2);
	    }
	}
	gdk_pixbuf::Pixbuf::new_from_bytes(&glib::Bytes::from(&bytes), pvec.colorspace, pvec.has_alpha, pvec.bits_per_sample, pvec.width as i32, pvec.height as i32, pvec.rowstride as i32)
    }
}

// operators for indexing points
impl Index<usize> for Pixvec {
    type Output = Vec<Color>;
    fn index<'a>(&'a self, i: usize) -> &'a Vec<Color> {
        &self.data[i]
    }
}

impl IndexMut<usize> for Pixvec {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut Vec<Color> {
        &mut self.data[i]
    }
}
