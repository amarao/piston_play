extern crate piston_window;
extern crate image;
use image::{RgbaImage, Rgba};
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
    color: image::Rgba<u8>
}
#[derive(Debug)]
enum Command {
    Count(u32),
    NewResolution(u32, u32),
    Continue
}

 #[derive(Debug)]
struct ControlCommand{
    command: Command
}
fn scale(buf: RgbaImage, old_x:u32, old_y:u32, new_x:u32, new_y:u32) -> RgbaImage{
    image::ImageBuffer::from_fn(new_x, new_y, |x, y| {
        if x < old_x && y < old_y {
            *(buf.get_pixel(x, y))
        }else{
            Rgba([255,255,255,255])
        }
    })
    // image::ImageBuffer::from_fn(new_x, new_y, |x, y| {
    //     Rgba([255,25,255,255])
    // })
}

fn main() {
    let mut x = 800;
    let mut y  = 600;
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
    let mut buf = image::ImageBuffer::from_fn(x, y, |_, __| { image::Rgba([255,255,255,255])});
    // println!("{:?}",ctrl.buf.get_pixel(0,0)[0]);
    // panic!("");
    let mut texture: G2dTexture = Texture::from_image(
                &mut texture_context,
                &buf,
                &TextureSettings::new()
            ).unwrap();
    let mut events = Events::new(EventSettings::new().lazy(false));
    let (draw_tx, draw_rx): (SyncSender<DrawCommand>, Receiver<DrawCommand>) = mpsc::sync_channel(128);
    let (control_tx, control_rx): (SyncSender<ControlCommand>, Receiver<ControlCommand>) = mpsc::sync_channel(8);
    thread::spawn(move ||{
            calc(draw_tx, control_rx, x, y)
    });
    while let Some(e) = events.next(&mut window) {
        if let Some(draw_event) = e.render_args() {
            if draw_event.draw_size[0] != x || draw_event.draw_size[1] != y {
                let new_x = draw_event.draw_size[0];
                let new_y = draw_event.draw_size[1];
                println!("Resolution change from {}x{} to {}x{}", x, y, new_x, new_y);
                control_tx.send(ControlCommand{command: Command::NewResolution(new_x, new_y)}).unwrap();
                while let Ok(_command) = draw_rx.try_recv(){}
                buf = scale(buf, x, y, new_x, new_y);
                x = new_x;
                y = new_y;
                control_tx.send(ControlCommand{command:Command::Continue}).unwrap();
                texture = Texture::from_image(
                    &mut texture_context,
                    &buf,
                    &TextureSettings::new()
                ).unwrap();
            }
            let mut c = 0;
            while let Ok(command) = draw_rx.try_recv(){
                if command.x > x || command.y > y {
                    panic!("Out of bound write: {}x{}", command.x, command.y)
                }else{
                    buf.put_pixel(command.x,command.y,command.color);
                }
                c+=1;
            }
            control_tx.send(ControlCommand{command: Command::Count(c)}).unwrap();
            texture.update(&mut texture_context, &buf).unwrap();
            window.draw_2d(&e, |c, g, device| {
                    texture_context.encoder.flush(device);
                    image(&texture, c.transform, g);
            });

        }
    }
}

fn calc(draw: SyncSender<DrawCommand>, command: Receiver<ControlCommand>, max_x:u32, max_y:u32){
    let mut cur_x = max_x;
    let mut cur_y = max_y;
    let mut rng = rand::thread_rng();
    let mut active = true;
    loop{
        loop {
            match command.try_recv() {
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    if !active {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
                Ok(cmd) => {
                    match cmd.command {
                        Command::Count(_counter) => {
                                // println!("counter: {}", counter);
                            },
                        Command::NewResolution(new_x, new_y) => {
                                println!("new resolution:{}x{}", new_x, new_y);
                                cur_x = new_x;
                                cur_y = new_y;
                                active = false;
                                continue;
                            },
                        Command::Continue => {
                            active = true;
                            println!("Continue to render.");
                            break;
                        }
                    }
                }
            }
        }
        let x = rng.gen_range(0,cur_x);
        let y = rng.gen_range(0,cur_y);
        let color = image::Rgba([rng.gen_range(0,255), rng.gen_range(0,255), rng.gen_range(0,255), rng.gen_range(0,255)]);
        if let Err(_)  = draw.send(DrawCommand{x, y, color}){
            break
        }
        thread::sleep(Duration::from_millis(1));
    }
}
