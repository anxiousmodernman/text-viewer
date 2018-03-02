#![allow(warnings)] // dev only
extern crate termion;
extern crate ropey;

use termion::clear;
use termion::cursor;
use termion::event;
use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead, Events, MouseTerminal};
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

    let mut gw: u32 = {
        if text.len_lines() > 9999 {
            5
        } else {
            4
        }
    };

    let mut pgr = Pager{
        buf: text,
        stdout: stdout,
        stdin: stdin.events(),
        top_pos: 0,
        gutter_width: gw,  // hardcoded
        w: w,
        h: h,
    };
    pgr.render_all();
}

impl<R: Read, W: Write> Pager<R, W> {

    /// Determine the total number of character cells available to our text 
    /// view, excluding the left gutter.
    fn text_area_chars(&self) -> usize {
        let term_area = (self.w * self.h) as usize;
        let gutter_area = (self.gutter_width * self.h as u32) as usize;
        term_area - gutter_area
    }

    /// Determine how many characters we can put on a line without wrapping in
    /// our text area, excluding the left gutter.
    fn text_area_line_width(&self) -> usize {
        (self.w as u32 - self.gutter_width) as usize
    }

    /// Writes a gutter with line number, followed by a single-line slice of our
    /// text buffer contents. If we need to overflow, an empty gutter is 
    /// written, and the caller must know what the next line will be. We return
    /// the number of text area lines required to write our line. If we only
    /// need one line, return one. If our line of text is very long and we 
    /// require 3, return 3, etc.
    fn write_text_line(&mut self, start_at: (u16, u16), line_no: u32, line: ropey::RopeSlice) -> usize {

        write!(self.stdout, " {}{}", cursor::Goto(start_at.0, start_at.1), line_no).unwrap();
        1
    }

    fn render_all(&mut self) {
        // max text per line, minus the side gutter
        let char_idx_start = self.buf.line_to_char(self.top_pos as usize);
        let mut pos = self.top_pos.clone();

        let mut editor_line: u16 = 0;

        for line in self.buf.lines() {
            let line_no = pos as usize + editor_line as usize + 1;
            //self.write_gutter(editor_line, pos as usize + editor_line as usize + 1);
            write!(self.stdout, " {}{}", cursor::Goto(1, editor_line+1), line_no).unwrap();
            editor_line += 1;
            if editor_line >= self.h {
                break;
            }
            write!(self.stdout, " {}{}", cursor::Goto(self.gutter_width as u16 + 1, editor_line+1), line).unwrap();
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
                            write!(self.stdout, "{}x", cursor::Goto(x, y)).unwrap();
                        }
                        _ => {}
                    }
                }
                _ => {
                }
            }
            self.stdout.flush().unwrap();
        }
    }
}

fn main() {
    let mut r = BufReader::new(File::open("examples/text.txt").unwrap());

    // had to move this AlternateScreen instantiation from inside render_all
    let mut stdout = MouseTerminal::from(AlternateScreen::from(stdout().into_raw_mode().unwrap()));
    let stdin = stdin();
    let (w, h) = terminal_size().unwrap();

    init(stdout, stdin, w, h, r);

}
