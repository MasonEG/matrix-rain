use std::io::{Write, stdout, Stdout, stdin};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use rand::prelude::ThreadRng;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::terminal_size;
use termion::clear;
use termion::event::Key;
use termion::input::TermRead;
use rand::Rng;

struct Raindrop {
    x: u16,
    y: u16,
}

fn gen_raindrop(dimensions: (u16, u16), rng: &mut ThreadRng, drops: &mut Vec<Raindrop>) {
    let rand_u16: u16 = rng.gen();
    let x_coord: u16 = rand_u16 % dimensions.0;
    drops.push(Raindrop{x: x_coord, y: 0});
}

fn draw(dimensions: (u16, u16), screen: &mut AlternateScreen<RawTerminal<Stdout>>, drops: &Vec<Raindrop>) {
    write!(screen, "{}", termion::clear::All);
    for d in drops {
        write!(screen, "{}d", termion::cursor::Goto(d.x, d.y)).unwrap();
    }
}

fn update(dimensions: (u16, u16), rng: &mut ThreadRng, drops: &mut Vec<Raindrop>) {
    // list of drop indexes I want to remove
    let mut indexes_to_remove: Vec<usize> = vec![];

    // update drop positions & find what drops I want to remove (the ones at the bottom of the screen)
    drops.iter_mut().enumerate().for_each(|(i, d)| {
        if d.y > dimensions.1 {
            indexes_to_remove.push(i);
        }
        d.y = d.y + 1;
    });

    // remove drops that have fallen
    for i in indexes_to_remove {
        drops.remove(i);
    }

    // gen some new drops
    for i in 0..10 {
        gen_raindrop(dimensions, rng, drops);
    }
}
fn main() {
    let dimensions = terminal_size().unwrap();
    // println!("width: {:?}, height: {:?}", dimensions.0, dimensions.1);
    let mut rng = rand::thread_rng();
    let mut raindrops: Vec<Raindrop> = vec![];
    for i in 0..10 {
        gen_raindrop(dimensions, &mut rng, &mut raindrops);
    }

    let (sendDrop, recieveDrop): (Sender<Vec<Raindrop>>, Receiver<Vec<Raindrop>>) = mpsc::channel();

    let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    let stdin = stdin();

    write!(screen, "{}", termion::cursor::Hide).unwrap();
    draw(dimensions, &mut screen, &raindrops);

    let thread_send_drop = sendDrop.clone();
    let drawThread = thread::spawn(move || {
        loop {
            update(dimensions, &mut rng, &mut raindrops);
            thread_send_drop.send(raindrops).unwrap();
            thread::sleep(Duration::from_millis(500));
        }
    });


    screen.flush().unwrap();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            Key::Char('u') => {
                update(dimensions, &mut rng, &mut raindrops); 
                draw(dimensions, &mut screen, &raindrops);
            }
            _ => {}
        }
    }
}