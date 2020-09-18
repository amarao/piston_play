extern crate piston_window;
extern crate image;
use image::{RgbaImage, Rgba};
use piston_window::*;
use piston;
use rand::Rng;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::sync::mpsc::{SyncSender, Sender, Receiver, TrySendError};

struct DrawCommand {
    x: u32,
    y: u32,
    color: image::Rgba<u8>
}
#[derive(Debug)]
enum Command {
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
}

fn main() {
    let mut x = 800;
    let mut y  = 600;
    let cpus = num_cpus::get();

    let (draw_tx, draw_rx): (SyncSender<DrawCommand>, Receiver<DrawCommand>) = mpsc::sync_channel(128);
    let mut control_txes: Vec<Sender<ControlCommand>> = Vec::new();
    for cpu in 1..cpus{
        let (control_tx, control_rx): (Sender<ControlCommand>, Receiver<ControlCommand>) = mpsc::channel();
        control_txes.push(control_tx);
        let thread_draw_tx = draw_tx.clone();
        thread::spawn(move ||{
                println!("Spawning thread for cpu {}", cpu);
                calc(thread_draw_tx, control_rx, x, y)
        });
    }

    let mut window: PistonWindow =
        WindowSettings::new("test", (x, y))
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };
    let mut buf = image::ImageBuffer::from_fn(x, y, |_, __| { image::Rgba([255,255,255,255])});
    let mut texture: G2dTexture = Texture::from_image(
                &mut texture_context,
                &buf,
                &TextureSettings::new()
            ).unwrap();
    let mut events = Events::new(EventSettings::new().lazy(false));
    let mut draw_per_sec = 16000;
    while let Some(e) = events.next(&mut window) {
        match e{
            piston::Event::Loop(piston::Loop::Idle(ref idle)) => {
                let start = std::time::Instant::now();
                let mut draws = (idle.dt*draw_per_sec as f64) as i32;
                if draws < 16 {
                    draws = 16;
                }
                for _count in 0..draws {
                    if let Ok(cmd) = draw_rx.try_recv(){
                        buf.put_pixel(cmd.x,cmd.y,cmd.color);
                    }else{
                        break;
                    }
                }
                let spent = (std::time::Instant::now() - start).as_secs_f64();
                let actual_draw_per_sec = ((draws as f64)/spent) as i32;
                if actual_draw_per_sec < 2 {
                    println!("oops, per sec: {}, draws: {}, sepnt: {}", actual_draw_per_sec, draws, spent);
                    continue;
                }
                if actual_draw_per_sec/draw_per_sec > 100 || draw_per_sec/actual_draw_per_sec > 100 {
                    println!("Changing pace. idle.dt:{}, spent: {}, old rate: {}, new rate: {}", idle.dt, spent, draw_per_sec, actual_draw_per_sec);
                    draw_per_sec = actual_draw_per_sec / 2;
                    if draw_per_sec < cpus as i32 {
                        draw_per_sec = cpus as i32 ;
                    }
                }

            }
            piston::Event::Loop(piston::Loop::AfterRender(_)) => {}
            piston::Event::Loop(piston::Loop::Render(_)) => {
                texture.update(&mut texture_context, &buf).unwrap();
                window.draw_2d(&e, |c, g, device| {
                        texture_context.encoder.flush(device);
                        image(&texture, c.transform, g);
                });
            }
            piston::Event::Loop(piston::Loop::Update(_)) => {

            }
            piston::Event::Input(piston::Input::Resize(piston::ResizeArgs{window_size:_, draw_size:[new_x, new_y]}), _) => {
                println!("Resize event: {}x{} (was {}x{})", new_x, new_y, x, y);
                for control_tx in &control_txes{
                    control_tx.send(ControlCommand{command: Command::NewResolution(new_x, new_y)}).unwrap();
                }
                println!("Purge queue.");
                while let Ok(_command) = draw_rx.try_recv(){};
                if let Ok(_) = draw_rx.try_recv(){
                    panic!("queue must be empty");
                }

                for control_tx in &control_txes{
                    println!("Sending continue.");
                    control_tx.send(ControlCommand{command:Command::Continue}).unwrap();
                }
                buf = scale(buf, x, y, new_x, new_y);
                x = new_x;
                y = new_y;

                texture = Texture::from_image(
                    &mut texture_context,
                    &buf,
                    &TextureSettings::new()
                ).unwrap();
            },
            piston::Event::Input(_, _) => {
            },
            ref something => {
                println!("Unexpected something: {:?}", something);
            },
        }
        window.event(&e);
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
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {return;},
                Ok(ControlCommand{command: Command::NewResolution(new_x, new_y)}) => {
                        println!("new resolution:{}x{}", new_x, new_y);
                        cur_x = new_x;
                        cur_y = new_y;
                        active = false;
                        continue;
                },
                Ok(ControlCommand{command:Command::Continue}) => {
                    active = true;
                    println!("Continue to render.");
                    break;
                }
            }
        }
        let x = rng.gen_range(0,cur_x);
        let y = rng.gen_range(0,cur_y);
        let color = image::Rgba([
            rng.gen_range(0,255),
            rng.gen_range(0,255),
            rng.gen_range(0,255),
            rng.gen_range(0,255)]
        );
        match draw.try_send(DrawCommand{x, y, color}){
            Err(TrySendError::Disconnected(_)) => {
                return;
            },
            Err(TrySendError::Full(_)) =>{
                continue;
            }
            Ok(_) => {}
        }
        // thread::sleep(Duration::from_millis(1));
    }
}
