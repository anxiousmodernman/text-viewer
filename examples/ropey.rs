#![allow(warnings)] // dev only
extern crate termion;
extern crate ropey;

use termion::clear;
use termion::cursor;
use termion::event;
use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead, Events};
use termion::raw::IntoRawMode;
use termion::screen::*;
use termion::terminal_size;
use termion::style;
use std::io;
use std::io::{Write, Read, Stdout, Stdin, Stderr, stdout, stdin, stderr, BufReader};
use std::fs::File;
use std::fmt::Debug;
use ropey::Rope;

struct Pager<R, W: Write>{
    buf: Rope,
    stdout: W,
    stdin: Events<R>,
    // the current logical line at the top of the buffer???
    top_pos: u32,
    gutter_width: u32,
    // height and width
    h: u16,
    w: u16,
}

fn init<W: Write, R: Read, RD: Read>(mut stdout: W, stdin: R, w: u16, h: u16, data: RD) {
    // clear the screen
    write!(stdout, "{}", clear::All).unwrap();

    let mut text = Rope::from_reader(data).unwrap();

    let mut pgr = Pager{
        buf: text,
        stdout: stdout,
        stdin: stdin.events(),
        top_pos: 0,
        gutter_width: 4,  // hardcoded
        w: w,
        h: h,
    };
    pgr.render_all();
}

//impl<R: Read + TermRead + Iterator<Item=Result<Event, std::io::Error>>, W: Write> Pager<R, W> {
impl<R: Read, W: Write> Pager<R, W> {
    fn render_all(&mut self) {
        // max text per line, minus the side gutter
        let line_chars = self.w as u32 - self.gutter_width;
        let char_idx_start = self.buf.line_to_char(self.top_pos as usize);
        let mut pos = self.top_pos.clone();

        let mut editor_line: u16 = 0;

        for line in self.buf.lines() {
            let line_no = pos as usize + editor_line as usize + 1;
            //self.write_gutter(editor_line, pos as usize + editor_line as usize + 1);
            write!(self.stdout, "{}{}", cursor::Goto(1, editor_line+1), line_no).unwrap();
            editor_line += 1;
            if editor_line > 10 {
                break;
            }
        }

        self.stdout.flush().unwrap();

        loop {
            let ev = self.stdin.next().unwrap().unwrap();
            match ev {
                Event::Key(Key::Char('q')) => {
                    write!(self.stdout, "{}{}{}goodbye", clear::All, style::Reset, cursor::Goto(1, 1)).unwrap();
                    self.stdout.flush().unwrap();
                    break;
                }
                Event::Mouse(me) => {
                    match me {
                        MouseEvent::Press(_, x, y) => {
                            write!(self.stdout, "{}x", termion::cursor::Goto(x, y)).unwrap();
                            self.stdout.flush().unwrap();
                        }
                        _ => {}
                    }
                }
                _ => {
                    //write!(self.stdout, "{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1)).unwrap();
                    //self.stdout.flush().unwrap();
                    //break;
                }

            }
        }
    }

    fn write_gutter(&mut self, go_to_line: u16, line_no: usize) {


    }
}

fn main() {
    let mut r = BufReader::new(File::open("examples/text.txt").unwrap());

    // had to move this AlternateScreen instantiation from inside render_all
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());
    let stdin = stdin();
    let (w, h) = terminal_size().unwrap();

    init(stdout, stdin, w, h, r);

}
