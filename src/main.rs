extern crate piston_window;
extern crate image as im;

use piston_window::*;
use piston::event_loop::Events;

fn main() {
    let x = 1920;
    let y  = 1080;
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("test", (x, y))
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
        .unwrap();
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };
    let mut events = Events::new(EventSettings::new().lazy(false));
    let buf = im::ImageBuffer::new(x, y);
    let mut texture: G2dTexture = Texture::from_image(
                &mut texture_context,
                &buf,
                &TextureSettings::new()
            ).unwrap();
    while let Some(e) = events.next(&mut window) {
        if let Some(_) = e.render_args() {
            texture.update(&mut texture_context, &buf).unwrap();
            window.draw_2d(&e, |c, g, device| {
                    texture_context.encoder.flush(device);
                    image(&texture, c.transform, g);
            });
        }
    }
}
