extern crate piston_window;
extern crate image as im;
use piston_window::*;
use piston::event_loop::Events;
use rand::Rng;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::sync::mpsc::{SyncSender, Receiver};

struct DrawCommand {
    x: u32,
    y: u32,
    color: im::Rgba<u8>
}

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
    let mut buf = im::ImageBuffer::new(x, y);
    let mut texture: G2dTexture = Texture::from_image(
                &mut texture_context,
                &buf,
                &TextureSettings::new()
            ).unwrap();
    let mut events = Events::new(EventSettings::new().lazy(false));
    let (tx,rx): (SyncSender<DrawCommand>, Receiver<DrawCommand>) = mpsc::sync_channel(128);
    thread::spawn(move ||{
            calc(tx,x,y)
    });
    while let Some(e) = events.next(&mut window) {
        if let Some(_) = e.render_args() {
            while let Ok(command) = rx.try_recv(){
                buf.put_pixel(command.x,command.y,command.color);
            }
            texture.update(&mut texture_context, &buf).unwrap();
            window.draw_2d(&e, |c, g, device| {
                    texture_context.encoder.flush(device);
                    image(&texture, c.transform, g);
            });

        }
    }
}

fn calc(tx: SyncSender<DrawCommand>, max_x:u32, max_y:u32){
    let mut rng = rand::thread_rng();
    loop{
        let x = rng.gen_range(0,max_x);
        let y = rng.gen_range(0,max_y);
        let color = im::Rgba([rng.gen_range(0,255), rng.gen_range(0,255), rng.gen_range(0,255), rng.gen_range(0,255)]);
        if let Err(_)  = tx.send(DrawCommand{x, y, color}) {break}
        thread::sleep(Duration::from_millis(1));
    }
}
