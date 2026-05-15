extern crate libc;

use libc::{
    atexit, exit, isatty, tcgetattr, tcsetattr, time, ioctl, STDIN_FILENO, TIOCGWINSZ, BRKINT, CS8,
    ECHO, ICANON, ICRNL, IEXTEN, ISIG, IXON, OPOST, TCSAFLUSH, VMIN, VTIME, winsize,
};
use std::env;
use std::ptr::null_mut;
use std::str;

/// Syntax highlight types
const HL_NORMAL: u8 = 0;
const HL_NONPRINT: u8 = 1;
const HL_COMMENT: u8 = 2;
const HL_MLCOMMENT: u8 = 3;
const HL_KEYWORD1: u8 = 4;
const HL_KEYWORD2: u8 = 5;
const HL_STRING: u8 = 6;
const HL_NUMBER: u8 = 7;
const HL_MATCH: u8 = 8;

const HL_HIGHLIGHT_STRINGS: u8 = 1 << 0;
const HL_HIGHLIGHT_NUMBERS: u8 = 1 << 1;

struct EditorSyntax {
    filematch: Vec<&'static str>,
    keywords: Vec<&'static str>,
    singleline_comment_start: &'static str,
    multiline_comment_start: &'static str,
    multiline_comment_end: &'static str,
    flags: i32,
}

struct ERow {
    idx: i32,
    size: i32,
    rsize: i32,
    chars: Vec<u8>,
    render: Vec<u8>,
    hl: Vec<u8>,
    hl_oc: i32,
}

struct HLColor {
    r: i32,
    g: i32,
    b: i32,
}

struct EditorConfig {
    cx: i32,
    cy: i32,
    rowoff: i32,
    coloff: i32,
    screenrows: i32,
    screencols: i32,
    numrows: i32,
    rawmode: i32,
    row: Vec<ERow>,
    dirty: i32,
    filename: Option<String>,
    statusmsg: String,
    statusmsg_time: u64,
    syntax: Option<&'static EditorSyntax>,
}

static mut E: EditorConfig = EditorConfig {
    cx: 0,
    cy: 0,
    rowoff: 0,
    coloff: 0,
    screenrows: 0,
    screencols: 0,
    numrows: 0,
    rawmode: 0,
    row: Vec::new(),
    dirty: 0,
    filename: None,
    statusmsg: String::new(),
    statusmsg_time: 0,
    syntax: None,
};

#[derive(PartialEq, Clone, Copy)]
enum KeyAction {
    KeyNull = 0,
    CtrlC = 3,
    CtrlD = 4,
    CtrlF = 6,
    CtrlH = 8,
    Tab = 9,
    CtrlL = 12,
    Enter = 13,
    CtrlQ = 17,
    CtrlS = 19,
    CtrlU = 21,
    Esc = 27,
    Backspace = 127,
    ArrowLeft = 1000,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    DelKey,
    HomeKey,
    EndKey,
    PageUp,
    PageDown,
}

fn editor_set_status_message(_: &str, args: std::fmt::Arguments) {
    unsafe {
        E.statusmsg = format!("{}", args);
        E.statusmsg_time = time(null_mut()) as u64;
    }
}

static mut orig_termios: libc::termios = libc::termios {
    c_iflag: 0,
    c_oflag: 0,
    c_cflag: 0,
    c_lflag: 0,
    c_line: 0,
    c_cc: [0; 32],
    c_ispeed: 0,
    c_ospeed: 0,
};

fn disable_raw_mode(fd: i32) {
    unsafe {
        if E.rawmode != 0 {
            tcsetattr(fd, TCSAFLUSH, &orig_termios);
            E.rawmode = 0;
        }
    }
}

extern "C" fn editor_at_exit() {
    disable_raw_mode(STDIN_FILENO);
}

fn enable_raw_mode(fd: i32) -> i32 {
    unsafe {
        if E.rawmode != 0 {
            return 0;
        }
        let tty = isatty(STDIN_FILENO);
        if tty == 0 {
            return -1;
        }
        atexit(editor_at_exit);
        if tcgetattr(fd, &mut orig_termios) == -1 {
            return -1;
        }
        let mut raw = orig_termios;
        raw.c_iflag &= !(BRKINT | ICRNL | IXON);
        raw.c_oflag &= !(OPOST);
        raw.c_cflag |= CS8;
        raw.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        raw.c_cc[VMIN] = 0;
        raw.c_cc[VTIME] = 1;

        if tcsetattr(fd, TCSAFLUSH, &raw) < 0 {
            return -1;
        }
        E.rawmode = 1;
        0
    }
}

fn editor_read_key(fd: i32) -> Result<KeyAction, i32> {
    let mut c: u8 = 0;
    let mut seq: [u8; 3] = [0; 3];

    unsafe {
        while libc::read(fd, &mut c as *mut u8 as *mut _, 1) != 1 {}
    }

    if c == KeyAction::Esc as u8 {
        let read_result;
        unsafe {
            read_result = libc::read(fd, &mut seq[0] as *mut u8 as *mut _, 1);
        }
        if read_result == 0 {
            return Ok(KeyAction::Esc);
        }
        unsafe {
            let read_result = libc::read(fd, &mut seq[1] as *mut u8 as *mut _, 1);
            if read_result == 0 {
                return Ok(KeyAction::Esc);
            }
        }

        if seq[0] == b'[' {
            if seq[1] >= b'0' && seq[1] <= b'9' {
                let read_result;
                unsafe {
                    read_result = libc::read(fd, &mut seq[2] as *mut u8 as *mut _, 1);
                }
                if read_result == 0 {
                    return Ok(KeyAction::Esc);
                }
                if seq[2] == b'~' {
                    match seq[1] {
                        b'3' => return Ok(KeyAction::DelKey),
                        b'5' => return Ok(KeyAction::PageUp),
                        b'6' => return Ok(KeyAction::PageDown),
                        _ => return Err(-1),
                    };
                }
            } else {
                match seq[1] {
                    b'A' => return Ok(KeyAction::ArrowUp),
                    b'B' => return Ok(KeyAction::ArrowDown),
                    b'C' => return Ok(KeyAction::ArrowRight),
                    b'D' => return Ok(KeyAction::ArrowLeft),
                    b'H' => return Ok(KeyAction::HomeKey),
                    b'F' => return Ok(KeyAction::EndKey),
                    _ => return Err(-1),
                };
            }
        } else if seq[0] == b'O' {
            match seq[1] {
                b'H' => return Ok(KeyAction::HomeKey),
                b'F' => return Ok(KeyAction::EndKey),
                _ => return Err(-1),
            }
        }
    } else {
        return Ok(match c {
            3 => KeyAction::CtrlC,
            4 => KeyAction::CtrlD,
            6 => KeyAction::CtrlF,
            8 => KeyAction::CtrlH,
            9 => KeyAction::Tab,
            12 => KeyAction::CtrlL,
            13 => KeyAction::Enter,
            17 => KeyAction::CtrlQ,
            19 => KeyAction::CtrlS,
            21 => KeyAction::CtrlU,
            27 => KeyAction::Esc,
            127 => KeyAction::Backspace,
            _ => KeyAction::KeyNull,
        });
    }
    Err(-1)
}

fn get_cursor_position(ifd: i32, ofd: i32, rows: &mut i32, cols: &mut i32) -> i32 {
    let mut buf: Vec<u8> = vec![0; 32];
    let mut i = 0;

    unsafe {
        if libc::write(ofd, b"\x1b[6n" as *const u8 as *const _, 4) != 4 {
            return -1;
        }
    }

    unsafe {
        while i < (buf.len() - 1) {
            if libc::read(ifd, &mut buf[i] as *mut u8 as *mut _, 1) != 1 {
                break;
            }
            if buf[i] == b'R' {
                break;
            }
            i += 1;
        }
        buf[i] = 0;
    }

    if buf[0] as char != '\x1b' || buf[1] as char != '[' {
        return -1;
    }

    let mut ps_vec: Vec<u8> = Vec::new();
    for x in 2..i {
        ps_vec.push(buf[x]);
    }
    let ps_str = str::from_utf8(&ps_vec).unwrap();
    let mut ps_iter = ps_str.split(';');
    if let (Some(r_str), Some(c_str)) = (ps_iter.next(), ps_iter.next()) {
        if let (Ok(r), Ok(c)) = (r_str.parse::<i32>(), c_str.parse::<i32>()) {
            *rows = r;
            *cols = c;
            return 0;
        }
    }
    -1
}

fn get_window_size(ifd: i32, ofd: i32, rows: &mut i32, cols: &mut i32) -> i32 {
    let mut ws: winsize = unsafe { std::mem::zeroed() };

    unsafe {
        if ioctl(1, TIOCGWINSZ, &mut ws) == -1 || ws.ws_col == 0 {
            let mut orig_row = 0;
            let mut orig_col = 0;
            if get_cursor_position(ifd, ofd, &mut orig_row, &mut orig_col) == -1 {
                return -1;
            }
            if libc::write(ofd, b"\x1b[999C\x1b[999B" as *const u8 as *const _, 12) != 12 {
                return -1;
            }
            if get_cursor_position(ifd, ofd, rows, cols) == -1 {
                return -1;
            }
            let sequence = format!("\x1b[{};{}H", orig_row, orig_col);
            if libc::write(ofd, sequence.as_bytes() as *const _ as *const _, sequence.len()) == -1 {
                return -1;
            }
            0
        } else {
            *cols = ws.ws_col as i32;
            *rows = ws.ws_row as i32;
            0
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: kilo <filename>");
        unsafe {
            exit(1);
        }
    }

    enable_raw_mode(STDIN_FILENO);
    editor_set_status_message(
        "HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find",
        format_args!(""),
    );

    loop {}
}