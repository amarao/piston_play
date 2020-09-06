extern crate piston_window;
extern crate image as im;
extern crate vecmath;

use piston_window::*;
use vecmath::*;

fn main() {
    let opengl = OpenGL::V3_2;
    let (width, height) = (300, 300);
    let mut window: PistonWindow =
        WindowSettings::new("piston: paint", (width, height))
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let mut canvas = im::ImageBuffer::new(width, height);
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };
    let mut texture: G2dTexture = Texture::from_image(
            &mut texture_context,
            &canvas,
            &TextureSettings::new()
        ).unwrap();

    let mut c = 0;
    while let Some(e) = window.next() {
        texture.update(&mut texture_context, &canvas).unwrap();
        window.draw_2d(&e, |c, g, device| {
            // Update texture before rendering.
            texture_context.encoder.flush(device);

            // clear([1.0; 4], g);
            image(&texture, c.transform, g);
        });
        canvas.put_pixel(c, c, im::Rgba([c as u8, c as u8, c as u8, 255]));
        c += 1;
        if c >= width || c >= height{
            c = 0;
        }
    }
}
