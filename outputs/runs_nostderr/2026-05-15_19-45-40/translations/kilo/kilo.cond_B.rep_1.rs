use std::env;
use std::time::SystemTime;
use std::mem;

const KILO_VERSION: &str = "0.0.1";
const STATUS_MSG_TIMEOUT: u64 = 5;
const HL_NORMAL: u8 = 0;
const HL_NONPRINT: u8 = 1;
const HL_COMMENT: u8 = 2; // Single line comment.
const HL_MLCOMMENT: u8 = 3; // Multi-line comment.
const HL_KEYWORD1: u8 = 4;
const HL_KEYWORD2: u8 = 5;
const HL_STRING: u8 = 6;
const HL_NUMBER: u8 = 7;
const HL_MATCH: u8 = 8; // Search match.

const HL_HIGHLIGHT_STRINGS: i32 = 1 << 0;
const HL_HIGHLIGHT_NUMBERS: i32 = 1 << 1;

#[derive(Debug, Clone)]
struct EditorSyntax {
    filematch: Vec<&'static str>,
    keywords: Vec<&'static str>,
    singleline_comment_start: &'static str,
    multiline_comment_start: &'static str,
    multiline_comment_end: &'static str,
    flags: i32,
}

#[derive(Debug, Clone)]
struct ERow {
    idx: usize,            // Row index in the file, zero-based.
    size: usize,           // Size of the row, excluding the null term.
    rsize: usize,          // Size of the rendered row.
    chars: Vec<u8>,        // Row content.
    render: Vec<u8>,       // Row content "rendered" for screen (for TABs).
    hl: Vec<u8>,           // Syntax highlight type for each character in render.
    hl_oc: bool,           // Row had open comment at end in last syntax highlight check.
}

#[derive(Debug, Clone)]
struct HLColor {
    r: u8,
    g: u8,
    b: u8,
}

struct EditorConfig {
    cx: usize,  // Cursor x position in characters
    cy: usize,  // Cursor y position in characters
    rowoff: usize,     // Offset of row displayed.
    coloff: usize,     // Offset of column displayed.
    screenrows: usize, // Number of rows that we can show
    screencols: usize, // Number of cols that we can show
    numrows: usize,    // Number of rows
    rawmode: bool,     // Is terminal raw mode enabled?
    row: Vec<ERow>,    // Rows
    dirty: bool,       // File modified but not saved.
    filename: Option<String>, // Currently open filename
    statusmsg: String,
    statusmsg_time: SystemTime,
    syntax: Option<EditorSyntax>,    // Current syntax highlight, or None.
}

static mut E: EditorConfig = EditorConfig {
    cx: 0,
    cy: 0,
    rowoff: 0,
    coloff: 0,
    screenrows: 0,
    screencols: 0,
    numrows: 0,
    rawmode: false,
    row: Vec::new(),
    dirty: false,
    filename: None,
    statusmsg: String::new(),
    statusmsg_time: SystemTime::UNIX_EPOCH,
    syntax: None,
};

extern "C" {
    fn tcgetattr(fd: i32, termios_p: *mut libc::termios) -> i32;
    fn tcsetattr(fd: i32, optional_actions: i32, termios_p: *const libc::termios) -> i32;
    fn isatty(fd: i32) -> i32;
    fn atexit(func: extern "C" fn()) -> i32;
    fn ioctl(fd: i32, request: libc::c_ulong, ...) -> i32;
    fn snprintf(str: *mut libc::c_char, size: usize, format: *const libc::c_char, ...) -> i32;
    fn strerror(errnum: i32) -> *mut libc::c_char;
}

mod libc {
    pub use libc::{
        termios, TCSAFLUSH, STDIN_FILENO, BRKINT, ICRNL, INPCK, ISTRIP, IXON, OPOST, CS8, ECHO, ICANON, IEXTEN, ISIG, VMIN, VTIME,
        c_ulong, c_char, c_void
    };

    #[link(name = "c")]
    extern "C" {
        pub fn read(fd: i32, buf: *mut c_void, count: usize) -> isize;
        pub fn exit(code: i32) -> !;
    }
}

const CTRL_C: i32 = 3;
const CTRL_D: i32 = 4;
const CTRL_F: i32 = 6;
const CTRL_H: i32 = 8;
const TAB: i32 = 9;
const CTRL_L: i32 = 12;
const ENTER: i32 = 13;
const CTRL_Q: i32 = 17;
const CTRL_S: i32 = 19;
const CTRL_U: i32 = 21;
const ESC: i32 = 27;
const BACKSPACE: i32 = 127;

const ARROW_LEFT: i32 = 1000;
const ARROW_RIGHT: i32 = 1001;
const ARROW_UP: i32 = 1002;
const ARROW_DOWN: i32 = 1003;
const DEL_KEY: i32 = 1004;
const HOME_KEY: i32 = 1005;
const END_KEY: i32 = 1006;
const PAGE_UP: i32 = 1007;
const PAGE_DOWN: i32 = 1008;

extern "C" fn disable_raw_mode(fd: i32) {
    unsafe {
        if E.rawmode {
            let mut orig_termios: libc::termios = mem::zeroed();
            tcgetattr(fd, &mut orig_termios);
            tcsetattr(fd, libc::TCSAFLUSH, &orig_termios);
            E.rawmode = false;
        }
    }
}

extern "C" fn editor_at_exit() {
    disable_raw_mode(libc::STDIN_FILENO);
}

fn enable_raw_mode(fd: i32) -> Result<(), &'static str> {
    unsafe {
        if E.rawmode { return Ok(()); }
        if isatty(libc::STDIN_FILENO) == 0 { return Err("Not a TTY"); }
        atexit(editor_at_exit);
        
        let mut orig_termios: libc::termios = mem::zeroed();
        tcgetattr(fd, &mut orig_termios);
        
        let mut raw = orig_termios;
        raw.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
        raw.c_oflag &= !(libc::OPOST);
        raw.c_cflag |= libc::CS8;
        raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
        raw.c_cc[libc::VMIN] = 0;
        raw.c_cc[libc::VTIME] = 1;
        
        if tcsetattr(fd, libc::TCSAFLUSH, &raw) < 0 { return Err("Could not enable raw mode."); }
        
        E.rawmode = true;
        Ok(())
    }
}

fn editor_read_key(fd: i32) -> i32 {
    let mut nread;
    let mut c: [u8; 1] = [0];
    while {
        nread = unsafe { libc::read(fd, c.as_mut_ptr() as *mut libc::c_void, 1) };
        nread == 0
    } {}
    
    if nread == -1 {
        unsafe { libc::exit(1) };
    }
    
    c[0] as i32
}

fn editor_set_status_message(fmt: &str) {
    unsafe {
        E.statusmsg = fmt.to_string();
        E.statusmsg_time = SystemTime::now();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: kilo <filename>");
        unsafe { libc::exit(1) };
    }

    unsafe {
        enable_raw_mode(libc::STDIN_FILENO).unwrap();
        editor_set_status_message("HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find");
        loop {
            // editorRefreshScreen();
            // editorProcessKeypress(STDIN_FILENO);
        }
    }
}