use image as im;
use piston_window as pw;
use piston;
use rand::Rng;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{SyncSender, Sender, Receiver, TryRecvError};
// use std::time::{Instant, Duration};
use piston_play:: {Buffer};

// const DRAW_BATCH_SIZE:usize = 255;
// #[derive(Debug)]
// struct DrawCommand {
//     num: usize,
//     x: [u32;DRAW_BATCH_SIZE],
//     y: [u32;DRAW_BATCH_SIZE],
//     color: [im::Rgba<u8>; DRAW_BATCH_SIZE]
// }


#[derive(Debug)]
enum Command {
    NewResolution(u32, u32, SyncSender<piston_play::Buffer>),
    NeedUpdate()
}


// fn process_draw_commands (allocated_time: Duration, rx: &Receiver<Command>, buf: &mut im::RgbaImage) -> u64{
//     let start = Instant::now();
//     while Instant::now() - start < allocated_time {
//         for _count in 0..1024 {
//             if let Ok(cmd) = rx.try_recv(){
//                 cnt += cmd.num as u64;
//                 for i in 0..cmd.num {
//                     buf.put_pixel(cmd.x[i],cmd.y[i],cmd.color[i]);
//                 }
//             }else{
//                 break;
//             }
//         }
//     }
//     cnt
// }

const MAX_THREADS:usize = 7;


#[derive(Default, Debug)]
struct ThreadCommands {
    control_tx: Option<SyncSender<Command>>,
    draw_rx: Option<Receiver<piston_play::Buffer>>,
    buf: Option<piston_play::Buffer>
}

fn get_updates(cpus: usize, control:&mut [ThreadCommands;MAX_THREADS]){
    for cpu in 0..cpus {
        match control[cpu].draw_rx.as_mut().unwrap().try_recv(){
            Ok(buf) =>{
                control[cpu].buf.replace(buf);
            }
            Err(TryRecvError::Empty) => {print!("!");}
            Err(TryRecvError::Disconnected) => {
                println!("disconnected in draw");
                continue;
            }

        }
    }
        // cnt += process_draw_commands(
        //     Duration::from_secs_f64(idle.dt),
        //     control[cpu].draw_rx.as_ref().unwrap(),
        //     control[cpu].buf.as_mut().unwrap().buf_mut_ref()
        // );
}

fn main() {
    let mut x = 800;
    let mut y  = 600;
    let cpus = std::cmp::min(num_cpus::get(), MAX_THREADS);
    let color_bases = [
        [255, 0, 0],
        [0, 255, 0],
        [255, 255, 0],
        [0, 0,255],
        [255, 0,255],
        [0, 255,255],
        [255, 255, 255]
    ];
    let mut control:[ThreadCommands;MAX_THREADS] = Default::default();

    for cpu in 0..cpus{
        let (control_tx, control_rx): (SyncSender<Command>, Receiver<Command>) = mpsc::sync_channel(1);
        let (draw_tx, draw_rx): (SyncSender<piston_play::Buffer>, Receiver<piston_play::Buffer>) = mpsc::sync_channel(1);
        control[cpu].control_tx = Some(control_tx);
        control[cpu].draw_rx = Some(draw_rx);
        control[cpu].buf = Some(Buffer::new(x, y/cpus as u32));
        thread::spawn(move ||{
                println!("Spawning thread for cpu {}", cpu);
                calc(draw_tx, control_rx, x, y/cpus as u32, color_bases[cpu])
        });
    }
    let mut window =
        pw::WindowSettings::new("test", (x, y))
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut events = pw::Events::new(
        (||{
            let mut settings = pw::EventSettings::new();
            settings.ups = 60;
            settings.max_fps = 60;
            settings
        })()
    );
    // let mut cnt = 0;


    while let Some(e) = events.next(&mut window) {
        match e{
            piston::Event::Loop(piston::Loop::Idle(_)) => {},
            piston::Event::Loop(piston::Loop::AfterRender(_)) => {
                for cpu in 0..cpus{
                    if let Err(_) = control[cpu].control_tx.as_ref().unwrap().try_send(Command::NeedUpdate()){
                        println!("update request errorr");
                    }
                }
            }
            piston::Event::Loop(piston::Loop::Render(_)) => {
                print!("*");
                let mut textures: Vec<piston_window::Texture<gfx_device_gl::Resources>> = Vec::new();
                for cpu in 0..cpus {
                    let texture = control[cpu].buf.as_ref().unwrap().as_texture(& mut window);
                    textures.push(texture);
                }
                window.draw_2d(
                    &e,
                    |context, graph_2d, _device| { //graph_2d -> https://docs.piston.rs/piston_window/gfx_graphics/struct.GfxGraphics.html
                        
                        // println!("transform: {:?}", context.transform);
                        // [
                        //      [0.0025, 0.0, -1.0],       ?, ? , ?
                        //      [0.0, -0.0033333333333333335, 1.0]  (some rotation),  Y-scale, Y offset (top is 1.0, bottom is -1)
                        //]
                        let mut transform = context.transform;
                        for cpu in 0..cpus {
                            transform[1][2] = 1.0 - 2.0 * cpu as f64 / cpus as f64 ;
                            pw::image(
                                &textures[cpu],
                                // context.reset().transform,
                                // [[0.00125, 0.0, -1.0], [0.0, -0.0016, 1.0]],  //left-top corner
                                // [[0.00125, 0.0, -1.0], [0.0, -0.0016666, 0.0]], //left-bottom corner
                                // [[0.00125, 0.0, 0.0], [0.0, -0.0016, 1.0]], //right-top corner
                                // [[0.00125, 0.0, 0.0], [0.0, -0.00166666, 0.0]], //right-bottom corner
                                transform,
                                graph_2d
                            );
                        }
                    }
                    
                );
                for _ in 0..cpus{
                    drop(textures.pop());
                }
                drop(textures);
            }
            piston::Event::Loop(piston::Loop::Update(_)) => {
                // println!("total idle time: {:.2}, pixels: {}, kpps: {:.1}", idle_time, cnt, cnt as f64/idle_time/1000.0);
                get_updates(cpus, &mut control);
            }
            piston::Event::Input(piston::Input::Resize(piston::ResizeArgs{window_size:_, draw_size:[new_x, new_y]}), _) => {
                println!("Resize event: {}x{} (was {}x{})", new_x, new_y, x, y);
                for cpu in 0..cpus{
                    let (new_draw_tx, new_draw_rx): (SyncSender<piston_play::Buffer>, Receiver<piston_play::Buffer>) = mpsc::sync_channel(1024);
                    control[cpu].control_tx.as_ref().unwrap().send(Command::NewResolution(
                            new_x, new_y/cpus as u32, new_draw_tx
                        )).unwrap();
                    control[cpu].draw_rx = Some(new_draw_rx);
                    control[cpu].buf.as_mut().unwrap().scale(new_x, new_y/cpus as u32);
                }

                x = new_x;
                y = new_y;
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

// fn gen_color(rng: & mut rand::rngs::ThreadRng, range: u8) -> u8{
//     if range > 0 {
//         rng.gen_range(0, range)
//     }
//     else{
//         0
//     }
// }



fn calc(mut draw: SyncSender<piston_play::Buffer>, command: Receiver<Command>, max_x:u32, max_y:u32, color_base:[u8;3]){
    let mut cur_x = max_x;
    let mut cur_y = max_y;
    let mut rng = rand::thread_rng();
    let mut cnt: u64 = 0;
    let mut start = std::time::Instant::now();
    let mut seed: u64 = 1;
    println!("new thread: {}x{}", max_x, max_y);
    let mut buf = piston_play::Buffer::new(max_x, max_y);
    loop{
        match command.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                println!("disconnected");
                return;
            },
            Ok(Command::NewResolution(new_x, new_y, new_draw)) => {
                    println!("new thread resolution:{}x{}", new_x, new_y);
                    cur_x = new_x;
                    cur_y = new_y;
                    buf.scale(new_x, new_y);
                    draw = new_draw;
            },
            Ok(Command::NeedUpdate()) => {
                if let Err(e) = draw.send(buf.clone()){
                    println!("buf send err: {}", e);
                    continue;
                }
                if start.elapsed().as_secs() >= 1 {
                    // println!("thread rate: {:.2} Mpps", cnt as f64 / start.elapsed().as_secs_f64()/1000.0/1000.0);
                    start = std::time::Instant::now();
                    cnt = 0;
                }
            }
            Err(_empty) => {
                seed = rng.gen_range(0, 2<<30);
                for _ in 0..1000 {
                    cnt += 1;
                    let val = seed ^ cnt;
                    buf.put_pixel(
                        (val % cur_x as u64) as u32,
                        (val % cur_y as u64) as u32,
                        im::Rgba([
                            if color_base[0] > 0 { (val % color_base[0] as u64) as u8 } else {0},
                            if color_base[1] > 0 { (val % color_base[1] as u64) as u8 } else {0},
                            if color_base[2] > 0 { (val % color_base[2] as u64) as u8 } else {0},
                            128,

                        ])
                    );
                    // buf.put_pixel(
                    //     rng.gen_range(0, cur_x),
                    //     rng.gen_range(0, cur_y),
                    //     im::Rgba([
                    //         gen_color(&mut rng, color_base[0]),
                    //         gen_color(&mut rng, color_base[1]),
                    //         gen_color(&mut rng, color_base[2]),
                    //         gen_color(&mut rng, 255),

                    //     ])
                    // );
                }
            }
        }
    }
}
