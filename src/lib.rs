use image as im;
use piston_window;
// use gfx_core;
use gfx_device_gl;

#[derive(Debug)]
pub struct Buffer{
    buf: im::ImageBuffer<im::Rgba<u8>,Vec<u8>>
}

impl Buffer{
    pub fn new(x:u32, y:u32)-> Self{
        Buffer{
            buf: im::ImageBuffer::from_fn(x, y, |_, __| { im::Rgba([255,255,255,255]) })
        }
    }
    
    pub fn put_pixel(&mut self, x: u32, y:u32, color: im::Rgba<u8>){
        self.buf.put_pixel(x, y, color);
    }

    pub fn scale(&mut self, new_x:u32, new_y:u32){
        let old_x = self.buf.width();
        let old_y = self.buf.height();
        let new_buf:im::ImageBuffer<im::Rgba<u8>,Vec<u8>> = im::ImageBuffer::from_fn(new_x, new_y, |x, y| {
            if x < old_x && y < old_y {
                *(self.buf.get_pixel(x, y))
            }else{
                im::Rgba([255,255,255,255])
            }
        });
        self.buf = new_buf;
    }

    pub fn buf_ref(&self) -> &im::ImageBuffer<im::Rgba<u8>,Vec<u8>>{
        &self.buf
    }

    pub fn buf_mut_ref(&mut self) -> &mut im::ImageBuffer<im::Rgba<u8>,Vec<u8>>{
        & mut self.buf
    }

    pub fn as_texture(
        &self,
        window: &mut piston_window::PistonWindow
    ) -> piston_window::Texture<gfx_device_gl::Resources>
    {
        // println!("buf x:{}, y: {}", self.x, self.y);
        let mut texture_context = window.create_texture_context();
        piston_window::Texture::from_image(
                &mut texture_context,
                &self.buf,
                &piston_window::TextureSettings::new()
            ).unwrap()
    }

    pub fn clone(&self) -> Self{
        Buffer{
            buf: self.buf.clone()
        }
    }
    pub fn replace(self, other: Self) -> Self{
        drop(self);
        return other;
    }
}