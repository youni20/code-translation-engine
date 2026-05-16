use std::env;
use std::io::{self, Write};
use std::os::unix::io::AsRawFd;
use std::sync::Once;
use std::cell::UnsafeCell;
use std::time::SystemTime;

const KILO_VERSION: &str = "0.0.1";

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

#[derive(Clone)]
struct EditorSyntax {
    filematch: Vec<&'static str>,
    keywords: Vec<&'static str>,
    singleline_comment_start: [char; 2],
    multiline_comment_start: [char; 3],
    multiline_comment_end: [char; 3],
    flags: i32,
}

struct ERow {
    idx: usize,
    size: usize,
    rsize: usize,
    chars: Vec<char>,
    render: Vec<char>,
    hl: Vec<u8>,
    hl_oc: bool,
}

struct EditorConfig {
    cx: usize,
    cy: usize,
    rowoff: usize,
    coloff: usize,
    screenrows: usize,
    screencols: usize,
    numrows: usize,
    rawmode: bool,
    row: Vec<ERow>,
    dirty: bool,
    filename: Option<String>,
    statusmsg: [u8; 80],
    statusmsg_time: Option<SystemTime>,
    syntax: Option<EditorSyntax>,
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
    statusmsg: [0; 80],
    statusmsg_time: None,
    syntax: None,
};

static INIT: Once = Once::new();
static mut SYNTAX_PTR: Option<*mut UnsafeCell<EditorSyntax>> = None;

fn get_hldb() -> &'static [EditorSyntax] {
    INIT.call_once(|| {
        let syntax = Box::into_raw(Box::new(UnsafeCell::new(EditorSyntax {
            filematch: vec![".c", ".h", ".cpp", ".hpp", ".cc"],
            keywords: vec![
                "auto", "break", "case", "continue", "default", "do", "else", "enum", "extern", "for",
                "goto", "if", "register", "return", "sizeof", "static", "struct", "switch", "typedef",
                "union", "volatile", "while", "NULL",
                "alignas", "alignof", "and", "and_eq", "asm", "bitand", "bitor", "class", "compl",
                "constexpr", "const_cast", "deltype", "delete", "dynamic_cast", "explicit", "export",
                "false", "friend", "inline", "mutable", "namespace", "new", "noexcept", "not", "not_eq",
                "nullptr", "operator", "or", "or_eq", "private", "protected", "public",
                "reinterpret_cast", "static_assert", "static_cast", "template", "this", "thread_local",
                "throw", "true", "try", "typeid", "typename", "virtual", "xor", "xor_eq",
                "int|", "long|", "double|", "float|", "char|", "unsigned|", "signed|", "void|", "short|",
                "auto|", "const|", "bool|",
            ],
            singleline_comment_start: ['/', '/'],
            multiline_comment_start: ['/', '*', '\0'],
            multiline_comment_end: ['*', '/', '\0'],
            flags: HL_HIGHLIGHT_STRINGS | HL_HIGHLIGHT_NUMBERS,
        })));
        unsafe {
            SYNTAX_PTR = Some(syntax);
        }
    });

    unsafe {
        std::slice::from_raw_parts((*SYNTAX_PTR.unwrap()).get(), 1)
    }
}

const HL_NORMAL: u8 = 0;
const HL_NONPRINT: u8 = 1;
const HL_COMMENT: u8 = 2;
const HL_MLCOMMENT: u8 = 3;
const HL_KEYWORD1: u8 = 4;
const HL_KEYWORD2: u8 = 5;
const HL_STRING: u8 = 6;
const HL_NUMBER: u8 = 7;
const HL_MATCH: u8 = 8;
const HL_HIGHLIGHT_STRINGS: i32 = 1 << 0;
const HL_HIGHLIGHT_NUMBERS: i32 = 1 << 1;

extern "C" {
    fn tcsetattr(fd: i32, actions: i32, termios_p: *const termios) -> i32;
    fn tcgetattr(fd: i32, termios_p: *mut termios) -> i32;
    fn isatty(fd: i32) -> i32;
}

#[repr(C)]
struct termios {
    c_iflag: u32,
    c_oflag: u32,
    c_cflag: u32,
    c_lflag: u32,
    c_cc: [u8; 32],
    c_ispeed: u32,
    c_ospeed: u32,
}

static mut orig_termios: termios = termios {
    c_iflag: 0,
    c_oflag: 0,
    c_cflag: 0,
    c_lflag: 0,
    c_cc: [0; 32],
    c_ispeed: 0,
    c_ospeed: 0,
};

fn disable_raw_mode(fd: i32) {
    unsafe {
        if E.rawmode {
            tcsetattr(fd, 2, &orig_termios);
            E.rawmode = false;
        }
    }
}

fn enable_raw_mode(fd: i32) -> Result<(), &'static str> {
    unsafe {
        if E.rawmode {
            return Ok(());
        }
        if isatty(fd) == 0 {
            return Err("Not a TTY");
        }
        at_exit(|| editor_at_exit());
        if tcgetattr(fd, &mut orig_termios) == -1 {
            return Err("Couldn't get terminal attributes");
        }

        let raw = termios {
            c_iflag: orig_termios.c_iflag & !(1 | 256 | 16 | 32 | 512),
            c_oflag: orig_termios.c_oflag & !1,
            c_cflag: orig_termios.c_cflag | 48,
            c_lflag: orig_termios.c_lflag & !(8 | 2 | 2048 | 1),
            c_cc: {
                let mut cc = orig_termios.c_cc;
                cc[6] = 0;
                cc[5] = 1;
                cc
            },
            c_ispeed: orig_termios.c_ispeed,
            c_ospeed: orig_termios.c_ospeed,
        };

        if tcsetattr(fd, 2, &raw) < 0 {
            return Err("Couldn't set terminal attributes");
        }
        E.rawmode = true;
        Ok(())
    }
}

fn at_exit<F: FnOnce()>(handler: F) {
    extern "C" {
        fn atexit(arg1: extern "C" fn()) -> i32;
    }

    extern "C" fn call_handler<F: FnOnce()>(handler: &mut Option<F>) {
        if let Some(h) = handler.take() {
            h();
        }
    }

    let handler_fn: extern "C" fn() = unsafe { std::mem::transmute(call_handler::<F> as *const ()) };

    unsafe {
        atexit(handler_fn);
    }
}

fn editor_read_key(fd: i32) -> io::Result<i32> {
    let mut nread;
    let mut c = [0];
    let mut seq = [0; 3];
    loop {
        nread = read(fd, c.as_mut_ptr() as *mut _, 1);
        if nread == 0 {
            continue;
        }
        if nread == -1 {
            return Err(io::Error::last_os_error());
        }
        match c[0] {
            27 => {
                if read(fd, seq.as_mut_ptr() as *mut _, 1) == 0 {
                    return Ok(KeyAction::Esc as i32);
                }
                if read(fd, unsafe { seq.as_mut_ptr().offset(1) } as *mut _, 1) == 0 {
                    return Ok(KeyAction::Esc as i32);
                }
                match seq[0] {
                    b'[' => match seq[1] {
                        b'A' => return Ok(KeyAction::ArrowUp as i32),
                        b'B' => return Ok(KeyAction::ArrowDown as i32),
                        b'C' => return Ok(KeyAction::ArrowRight as i32),
                        b'D' => return Ok(KeyAction::ArrowLeft as i32),
                        b'H' => return Ok(KeyAction::HomeKey as i32),
                        b'F' => return Ok(KeyAction::EndKey as i32),
                        _ => {}
                    },
                    b'O' => match seq[1] {
                        b'H' => return Ok(KeyAction::HomeKey as i32),
                        b'F' => return Ok(KeyAction::EndKey as i32),
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => return Ok(c[0] as i32),
        }
    }
}

#[inline]
fn is_separator(c: char) -> bool {
    c.is_whitespace() || ",.()+-/*=~%[];".contains(c)
}

impl ERow {
    fn has_open_comment(&self) -> bool {
        self.hl.last().map_or(false, |&hl| {
            hl == HL_MLCOMMENT
                && !self.render.ends_with(&['*', '/'])
        })
    }

    fn update_syntax(&mut self) {
        self.hl = vec![HL_NORMAL; self.rsize];
        unsafe {
            if E.syntax.is_none() {
                return;
            }
    
            let syntax = E.syntax.as_ref().unwrap();
            let keywords = &syntax.keywords;
            let scs = syntax.singleline_comment_start;
            let mcs = syntax.multiline_comment_start;
            let mce = syntax.multiline_comment_end;
    
            let mut i = 0;
            let mut prev_sep = true;
            let mut in_string = None;
            let mut in_comment = false;
    
            if self.idx > 0 && E.row[self.idx - 1].has_open_comment() {
                in_comment = true;
            }
    
            while i < self.render.len() {
                let c = self.render[i];
    
                if prev_sep && self.render[i..].starts_with(&scs) {
                    self.hl[i..].fill(HL_COMMENT);
                    return;
                }
    
                if in_comment {
                    self.hl[i] = HL_MLCOMMENT;
                    if self.render[i..].starts_with(&mce) {
                        self.hl[i + 1] = HL_MLCOMMENT;
                        i += mce.len();
                        in_comment = false;
                        prev_sep = true;
                        continue;
                    } else {
                        prev_sep = false;
                        i += 1;
                        continue;
                    }
                } else if self.render[i..].starts_with(&mcs) {
                    self.hl[i] = HL_MLCOMMENT;
                    self.hl[i + 1] = HL_MLCOMMENT;
                    i += mcs.len();
                    in_comment = true;
                    prev_sep = false;
                    continue;
                }
    
                if let Some(quote) = in_string {
                    self.hl[i] = HL_STRING;
                    if c == '\\' && i + 1 < self.render.len() {
                        self.hl[i + 1] = HL_STRING;
                        i += 2;
                        prev_sep = false;
                        continue;
                    }
                    if c == quote {
                        in_string = None;
                    }
                    i += 1;
                    continue;
                } else if c == '"' || c == '\'' {
                    in_string = Some(c);
                    self.hl[i] = HL_STRING;
                    i += 1;
                    prev_sep = false;
                    continue;
                }
    
                if !c.is_ascii() {
                    self.hl[i] = HL_NONPRINT;
                    i += 1;
                    prev_sep = false;
                    continue;
                }
    
                if (c.is_ascii_digit() && (prev_sep || self.hl[i - 1] == HL_NUMBER))
                    || (c == '.' && i > 0 && self.hl[i - 1] == HL_NUMBER)
                {
                    self.hl[i] = HL_NUMBER;
                    i += 1;
                    prev_sep = false;
                    continue;
                }
    
                if prev_sep {
                    for keyword in keywords {
                        let is_kw2 = keyword.ends_with('|');
                        let kw_len = if is_kw2 { keyword.len() - 1 } else { keyword.len() };
                        if self.render[i..].iter().collect::<String>().starts_with(keyword) 
                            && is_separator(self.render[i + kw_len]) {
                            let hl_type = if is_kw2 { HL_KEYWORD2 } else { HL_KEYWORD1 };
                            self.hl[i..i + kw_len].fill(hl_type);
                            i += kw_len;
                            prev_sep = false;
                            continue;
                        }
                    }
                }
    
                prev_sep = is_separator(c);
                i += 1;
            }
        }
    }
}

fn editor_syntax_to_color(hl: u8) -> i32 {
    match hl {
        HL_COMMENT | HL_MLCOMMENT => 36,
        HL_KEYWORD1 => 33,
        HL_KEYWORD2 => 32,
        HL_STRING => 35,
        HL_NUMBER => 31,
        HL_MATCH => 34,
        _ => 37,
    }
}

fn editor_select_syntax_highlight(filename: &str) {
    let hldb = get_hldb();
    for syntax in hldb.iter() {
        for pattern in &syntax.filematch {
            if filename.ends_with(pattern) || pattern.starts_with('.') {
                unsafe {
                    E.syntax = Some(syntax.clone());
                }
                return;
            }
        }
    }
}

fn update_window_size() -> io::Result<()> {
    let (cols, rows) = (80, 24);
    unsafe {
        E.screenrows = rows - 2;
        E.screencols = cols;
    }
    
    Ok(())
}

fn handle_sigwinch(_signal: i32) {
    if update_window_size().is_ok() {
        editor_refresh_screen();
    }
}

fn init_editor() {
    unsafe {
        E = EditorConfig {
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
            statusmsg: [0; 80],
            statusmsg_time: None,
            syntax: None,
        };
    }
    update_window_size().unwrap();
    unsafe {
        signal(28, handle_sigwinch);
    }
}

fn signal(signal: i32, handler: extern "C" fn(i32)) -> i32 {
    extern "C" {
        fn signal(arg1: i32, arg2: extern "C" fn(i32)) -> i32;
    }
    unsafe { signal(signal, handler) }
}

fn read(fd: i32, buf: *mut u8, count: usize) -> i32 {
    extern "C" {
        fn read(arg1: i32, arg2: *mut u8, arg3: usize) -> i32;
    }
    unsafe { read(fd, buf, count) }
}

fn editor_refresh_screen() {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write!(handle, "\x1b[?25l\x1b[H").unwrap();

    unsafe {
        for y in 0..E.screenrows {
            let file_row = E.rowoff + y;

            if file_row >= E.numrows {
                if E.numrows == 0 && y == E.screenrows / 3 {
                    let welcome = format!("Kilo editor -- version {}\x1b[0K\r\n", KILO_VERSION);
                    let padding = (E.screencols - welcome.len()) / 2;
                    if padding > 0 {
                        write!(handle, "~").unwrap();
                        for _ in 0..padding - 1 {
                            write!(handle, " ").unwrap();
                        }
                    }
                    write!(handle, "{}", welcome).unwrap();
                } else {
                    write!(handle, "~\x1b[0K\r\n").unwrap();
                }
                continue;
            }

            let row = &E.row[file_row];
            let len = row.rsize - E.coloff;
            let mut current_color = -1;
            let c = row.render[E.coloff..].iter();
            let hl = &row.hl[E.coloff..];
            for (&ch, &hl_type) in c.zip(hl.iter()) {
                if hl_type == HL_NORMAL {
                    if current_color != -1 {
                        write!(handle, "\x1b[39m").unwrap();
                        current_color = -1;
                    }
                    write!(handle, "{}", ch).unwrap();
                } else {
                    let color = editor_syntax_to_color(hl_type);
                    if color != current_color {
                        write!(handle, "\x1b[{}m", color).unwrap();
                        current_color = color;
                    }
                    write!(handle, "{}", ch).unwrap();
                }
            }
            write!(handle, "\x1b[39m").unwrap();
            write!(handle, "\x1b[0K").unwrap();
            write!(handle, "\r\n").unwrap();
        }

        write!(handle, "\x1b[0K").unwrap();
        write!(handle, "\x1b[7m").unwrap();
        write!(
            handle,
            "{} - {} lines {}",
            &E.filename.as_ref().unwrap_or(&"[No Name]".to_string()),
            E.numrows,
            if E.dirty { "(modified)" } else { "" }
        ).unwrap();
        for _ in 0..E.screencols {
            write!(handle, " ").unwrap();
        }
        write!(handle, "\x1b[0m\r\n").unwrap();
        write!(handle, "\x1b[?25h").unwrap();

        let cx = E.cx + 1;
        let cy = E.cy + 1;
        write!(handle, "\x1b[{};{}H", cy, cx).unwrap();
        handle.flush().unwrap();
    }
}

fn editor_set_status_message(msg: &str) {
    unsafe {
        E.statusmsg[..msg.len()].copy_from_slice(msg.as_bytes());
        E.statusmsg_time = Some(SystemTime::now());
    }
}

extern "C" fn editor_at_exit() {
    unsafe {
        disable_raw_mode(0);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: kilo <filename>");
        return;
    }

    init_editor();
    editor_select_syntax_highlight(&args[1]);

    if let Err(err) = enable_raw_mode(io::stdin().as_raw_fd()) {
        eprintln!("Error setting raw mode: {}", err);
        return;
    }

    editor_set_status_message("HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find");

    while {
        editor_refresh_screen();
        editor_read_key(io::stdin().as_raw_fd()).is_ok()
    } {}
}