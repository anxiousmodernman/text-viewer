#![allow(warnings)] // dev only
extern crate termion;
extern crate ropey;

use termion::clear;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::*;
use termion::terminal_size;
use std::io;
use std::io::{Write, Read, Stdout, Stdin, Stderr, stdout, stdin, stderr, BufReader};
use std::fs::File;
use std::fmt::Debug;
use ropey::Rope;

//fn write_alt_screen_msg<W: Write>(screen: &mut W) {
//    write!(screen, "{}{}Welcome to the alternate screen.{}Press '1' to switch to the main screen or '2' to switch to the alternate screen.{}Press 'q' to exit (and switch back to the main screen).",
//           termion::clear::All,
//           termion::cursor::Goto(1, 1),
//           termion::cursor::Goto(1, 3),
//           termion::cursor::Goto(1, 4)).unwrap();
//}

struct Pager<R, W: Write>{
    buf: Rope,
    stdout: W,
    stdin: R,
    top_pos: u32,
    gutter_width: u32,
}

struct LogicalLine{
    line_no: u32,
}

fn init<W: Write, R: Read, RD: Read>(mut stdout: W, stdin: R, w: u16, h: u16, data: RD) {
    // clear the screen
    write!(stdout, "{}", clear::All).unwrap();
}

//impl Pager<R>{
//    pub fn from_reader<R>(r: R) -> io::Result<Pager<R>> where R: Read {
//        let pgr = Pager{
//            buf: Rope::from_reader(r)?,
//            stdout: stdout(),
//            stderr: stderr(),
//            stdin: stdin().keys(),
//            top_pos: 1,
//            gutter_width: 4,
//        };
//        Ok(pgr)
//    }
//
//    //pub fn event_loop(&mut self) -> io::Result<()> {
//
//    //    loop {
//    //        // wait for keys; get one byte, a Char, probably.
//    //        let c = self.stdin.next().unwrap().unwrap();
//    //        match c {
//    //            Key::Char('q') | Key::Ctrl('c') => {
//    //                return Ok(());    
//    //            },
//    //            _ => {}
//    //        }
//    //          self.stdout.flush().unwrap();
//    //    }
//
//    //    Ok(())
//    //}
//}


fn main() {
    let mut r = BufReader::new(File::open("examples/text.txt").unwrap());

    let mut stdout = stdout();
    let stdin = stdin();
    let (w, h) = terminal_size().unwrap();

    init(stdout, stdin, w, h, r);

}
