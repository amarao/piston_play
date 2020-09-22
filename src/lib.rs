use image as im;

#[derive(Debug)]
pub struct Buffer{
    x: u32,
    y: u32,
    buf: im::ImageBuffer<im::Rgba<u8>,Vec<u8>>
}

impl Buffer{
    pub fn new(x:u32, y:u32)-> Self{
        Buffer{
            x: x,
            y: y,
            buf: im::ImageBuffer::from_fn(x, y, |_, __| { im::Rgba([255,255,255,255]) })
        }
    }


    pub fn scale(&mut self, new_x:u32, new_y:u32){
        let new_buf:im::ImageBuffer<im::Rgba<u8>,Vec<u8>> = im::ImageBuffer::from_fn(new_x, new_y, |x, y| {
            if x < self.x && y < self.y {
                *(self.buf.get_pixel(x, y))
            }else{
                im::Rgba([255,255,255,255])
            }
        });
        self.buf = new_buf;
        self.x = new_x;
        self.y = new_y;
    }

    pub fn buf_ref(&self) -> &im::ImageBuffer<im::Rgba<u8>,Vec<u8>>{
        &self.buf
    }

    pub fn buf_mut_ref(&mut self) -> &mut im::ImageBuffer<im::Rgba<u8>,Vec<u8>>{
        & mut self.buf
    }
}
