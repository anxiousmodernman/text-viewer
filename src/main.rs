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
use std::ops::Range;
use std::iter::Iterator;
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

        let takes_up = line.len_chars() / self.text_area_line_width();

        write!(self.stdout, " {}{}", cursor::Goto(start_at.0, start_at.1), line_no).unwrap();
        takes_up
    }


    fn render_all(&mut self) {
        // max text per line, minus the side gutter
        let char_idx_start = self.buf.line_to_char(self.top_pos as usize);
        let mut pos = self.top_pos.clone();

        let mut editor_line: u16 = 0;

        for line in self.buf.lines() {
            let line_no = pos as usize + editor_line as usize + 1;
            let takes_up = line_occupies(line.len_chars(), self.text_area_line_width());
            if (editor_line as usize + takes_up) > self.h as usize {
                // We will overflow our editor. Stop printing after this line.

            }
            if line.len_chars() >= self.text_area_line_width() {
                let mut ranges = get_ranges(line.len_chars(), self.text_area_line_width());
                for i in 0..ranges.len() {
                    editor_line += 1;
                    if editor_line >= self.h {
                        break;
                    }
                    // write the first gutter with a line number
                    if i == 0 {
                        write_gutter(&mut self.stdout, line_no, pos as u16 + editor_line as u16 + 1, false);
                    } else {
                        // passing 0 omits line number from gutter
                        write_gutter(&mut self.stdout, 0, pos as u16 + editor_line as u16 + 1, false);
                    }
                    // we're splitting over multiple editor lines, so we slice
                    let slc = line.slice(ranges[i].clone());
                    write!(self.stdout, " {}{}", cursor::Goto(self.gutter_width as u16 + 1, editor_line), slc).unwrap();
    
                }
            } else {
                editor_line += 1;
                if editor_line >= self.h {
                    break;
                }
                // text fits on one line, write a gutter with a line number
                write_gutter(&mut self.stdout, line_no, pos as u16 + editor_line as u16 + 1, false);
                write!(self.stdout, " {}{}", cursor::Goto(self.gutter_width as u16 + 1, editor_line), line).unwrap();
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

fn write_gutter<W: Write>(w: &mut W, line_no: usize, editor_line: u16, flush_buffer: bool) {
        if line_no == 0 {
            // pass 0 for a blank gutter when overflowing lines.
            write!(w, " {} ", cursor::Goto(1, editor_line+1)).unwrap();
        } else {
            write!(w, " {}{}", cursor::Goto(1, editor_line+1), line_no).unwrap();
        }
        if flush_buffer {
            w.flush().unwrap();
        };
}

/// Compute how many lines of the editor we require to display a line of text.
/// If greater than 1, that means we'll need to slice our line into N segments
/// for wrapping. We always return at least 1.
fn line_occupies(line_len: usize, editor_width: usize) -> usize {
    if line_len <= editor_width {
        1
    } else {
        let mut n = line_len / editor_width;
        if (line_len % editor_width) != 0 {
            n += 1;
        }
        n
    }
}

fn get_ranges(line_len: usize, editor_width: usize) -> Vec<Range<usize>> {
    let mut ranges = Vec::with_capacity(line_occupies(line_len, editor_width));
    if line_len <= editor_width {
        return vec![(0..line_len)];
    } else {
        let mut n = line_len / editor_width;
        if (line_len % editor_width) != 0 {
            n += 1;
            for i in (0..n) {
                if i == (n - 1) {
                    let start = i * editor_width; 
                    let end = (i * editor_width) + (line_len % editor_width);
                    let r = (start..end);
                    ranges.push(r);
                } else {
                    let start = i * editor_width; 
                    let end = (i * editor_width) + editor_width;
                    let r = (start..end);
                    ranges.push(r);
                }
            }
            return ranges;
        } else {
            for i in (0..n) {
                let start = i * editor_width; 
                let end = (i * editor_width) + editor_width;
                let r = (start..end);
                ranges.push(r);
            }
            return ranges;
        }
        return ranges;
    }
    return ranges;
}

fn main() {
    let mut r = BufReader::new(File::open("examples/text.txt").unwrap());

    // had to move this AlternateScreen instantiation from inside render_all
    let mut stdout = MouseTerminal::from(AlternateScreen::from(stdout().into_raw_mode().unwrap()));
    let stdin = stdin();
    let (w, h) = terminal_size().unwrap();

    init(stdout, stdin, w, h, r);

}

#[test]
fn test_get_ranges() {
    assert_eq!(get_ranges(17, 7), vec![(0..7), (7..14), (14..17)]);
    assert_eq!(get_ranges(14, 7), vec![(0..7), (7..14)]);
    assert_eq!(get_ranges(1, 7), vec![(0..1)]);
    assert_eq!(get_ranges(0, 7), vec![(0..0)]); // is this the behavior we want?
}

#[test]
fn test_line_occupies() {
    let line = 100;
    let editor = 50;
    assert_eq!(2, line_occupies(line, editor));
    assert_eq!(3, line_occupies(101, 50));
    assert_eq!(1, line_occupies(0, 50));
    assert_eq!(5, line_occupies(201, 50));

}

#[test]
fn do_math() {
    // How does division work in Rust?
    let answer = 100 as usize / 33 as usize;
    assert_eq!(answer, 3);
    let answer = 100 as usize % 33 as usize;
    assert_eq!(answer, 1);
    let answer = 16 as usize / 9 as usize;
    assert_eq!(answer, 1);
    let answer = 16 as usize % 9 as usize;
    assert_eq!(answer, 7);
    // Okay, cool.
}
