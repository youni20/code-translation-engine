use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::mem;
use std::process;
use std::str;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::ffi::CString;
use std::os::raw;
use std::cmp::{min, max};

const KILO_VERSION: &str = "0.0.1";

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

#[derive(Clone)]
pub struct EditorSyntax {
    pub filematch: &'static [&'static str],
    pub keywords: &'static [&'static str],
    pub singleline_comment_start: &'static str,
    pub multiline_comment_start: &'static str,
    pub multiline_comment_end: &'static str,
    pub flags: i32,
}

#[derive(Default, Clone)]
pub struct ERow {
    pub idx: usize,
    pub size: usize,
    pub rsize: usize,
    pub chars: Vec<u8>,
    pub render: Vec<u8>,
    pub hl: Vec<u8>,
    pub hl_oc: bool,
}

#[derive(Clone)]
pub struct HLColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone)]
pub struct EditorConfig {
    pub cx: usize,
    pub cy: usize,
    pub rowoff: usize,
    pub coloff: usize,
    pub screenrows: usize,
    pub screencols: usize,
    pub numrows: usize,
    pub row: Vec<ERow>,
    pub dirty: bool,
    pub filename: Option<String>,
    pub statusmsg: String,
    pub statusmsg_time: SystemTime,
    pub syntax: Option<EditorSyntax>,
    pub rawmode: bool,
}

static mut EDITOR: EditorConfig = EditorConfig {
    cx: 0,
    cy: 0,
    rowoff: 0,
    coloff: 0,
    screenrows: 0,
    screencols: 0,
    numrows: 0,
    row: vec![],
    dirty: false,
    filename: None,
    statusmsg: String::new(),
    statusmsg_time: UNIX_EPOCH,
    syntax: None,
    rawmode: false,
};

#[derive(PartialEq)]
pub enum KeyAction {
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

impl From<i32> for KeyAction {
    fn from(item: i32) -> KeyAction {
        use KeyAction::*;
        match item {
            3 => CtrlC,
            4 => CtrlD,
            6 => CtrlF,
            8 => CtrlH,
            9 => Tab,
            12 => CtrlL,
            13 => Enter,
            17 => CtrlQ,
            19 => CtrlS,
            21 => CtrlU,
            27 => Esc,
            127 => Backspace,
            1000 => ArrowLeft,
            1001 => ArrowRight,
            1002 => ArrowUp,
            1003 => ArrowDown,
            1004 => DelKey,
            1005 => HomeKey,
            1006 => EndKey,
            1007 => PageUp,
            _ => KeyNull,
        }
    }
}

fn editor_set_status_message(msg: &str) {
    unsafe {
        EDITOR.statusmsg = msg.to_string();
        EDITOR.statusmsg_time = SystemTime::now();
    }
}

extern "C" {
    fn tcsetattr(fd: i32, optional_actions: i32, termios_p: *const raw::c_void) -> i32;
    fn tcgetattr(fd: i32, termios_p: *mut raw::c_void) -> i32;
    fn atexit(func: extern "C" fn()) -> i32;
    fn isatty(fd: i32) -> i32;
    static STDIN_FILENO: i32;
    fn read(fd: i32, buf: *mut u8, count: usize) -> isize;
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn ioctl(fd: i32, request: u64, ...) -> i32;
    fn sscanf(str: *const i8, format: *const i8, ...) -> i32;
}

fn disable_raw_mode(fd: i32) {
    unsafe {
        if EDITOR.rawmode {
            let orig_termios: raw::c_void = mem::zeroed();
            tcsetattr(fd, 0, &orig_termios);
            EDITOR.rawmode = false;
        }
    }
}

extern "C" fn editor_at_exit() {
    disable_raw_mode(0);
}

fn enable_raw_mode(fd: i32) -> io::Result<()> {
    unsafe {
        if EDITOR.rawmode {
            return Ok(());
        }

        if isatty(0) == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "Not a tty"));
        }

        atexit(editor_at_exit);

        let mut orig_termios: raw::termios = mem::zeroed();
        if tcgetattr(fd, &mut orig_termios as *mut _ as *mut raw::c_void) == -1 {
            return Err(io::Error::last_os_error());
        }

        let mut raw = orig_termios;
        raw.c_lflag &= !(raw::ECHO | raw::ICANON | raw::ISIG | raw::IEXTEN);
        raw.c_iflag &= !(raw::IXON | raw::ICRNL | raw::BRKINT | raw::INPCK | raw::ISTRIP);
        raw.c_cflag |= raw::CS8;
        raw.c_oflag &= !(raw::OPOST);

        if tcsetattr(fd, raw::TCSAFLUSH, &raw as *const _ as *const raw::c_void) < 0 {
            return Err(io::Error::last_os_error());
        }

        EDITOR.rawmode = true;
        Ok(())
    }
}

fn editor_read_key(fd: i32) -> i32 {
    let mut nread = 0;
    let mut c: [u8; 1] = [0];
    while nread == 0 {
        nread = unsafe { read(fd, c.as_mut_ptr() as *mut _, 1) };
        if nread == -1 {
            process::exit(1);
        }
    }

    if c[0] == 27 {
        let mut seq: [u8; 3] = [0; 3];
        if unsafe { read(fd, seq.as_mut_ptr() as *mut _, 1) } == 0 {
            return 27; // ESC
        }
        if unsafe { read(fd, seq.as_mut_ptr().add(1) as *mut _, 1) } == 0 {
            return 27; // ESC
        }

        match seq[0] {
            b'[' => {
                if (b'0'..=b'9').contains(&seq[1]) {
                    if unsafe { read(fd, seq.as_mut_ptr().add(2) as *mut _, 1) } == 0 {
                        return 27;
                    }
                    if seq[2] == b'~' {
                        return match seq[1] {
                            b'3' => KeyAction::DelKey as i32,
                            b'5' => KeyAction::PageUp as i32,
                            b'6' => KeyAction::PageDown as i32,
                            _ => 27,
                        }
                    }
                } else {
                    return match seq[1] {
                        b'A' => KeyAction::ArrowUp as i32,
                        b'B' => KeyAction::ArrowDown as i32,
                        b'C' => KeyAction::ArrowRight as i32,
                        b'D' => KeyAction::ArrowLeft as i32,
                        b'H' => KeyAction::HomeKey as i32,
                        b'F' => KeyAction::EndKey as i32,
                        _ => 27,
                    }
                }
            }
            b'O' => {
                return match seq[1] {
                    b'H' => KeyAction::HomeKey as i32,
                    b'F' => KeyAction::EndKey as i32,
                    _ => 27,
                }
            }
            _ => {}
        }
    }

    c[0] as i32
}

fn get_cursor_position(ifd: i32, ofd: i32, rows: &mut i32, cols: &mut i32) -> i32 {
    let mut buf = [0u8; 32];
    let mut i = 0;

    if unsafe { write(ofd, "\x1b[6n".as_bytes().as_ptr() as *const _, 4) } != 4 {
        return -1;
    }

    while i < (buf.len() - 1) {
        if unsafe { read(ifd, &mut buf[i..i+1] as *mut _ as *mut _, 1) } == 1 {
            if buf[i] == b'R' {
                break;
            }
            i += 1;
        }
    }

    buf[i] = 0;
    let response = str::from_utf8(&buf[0..i]).unwrap_or("");
    if response.starts_with("\x1b[") && !response[2..].contains('R') {
        unsafe {
            let res = CString::new(response).unwrap();
            sscanf(res.as_ptr(), "%d;%d".as_ptr() as *const i8, rows, cols)
        }
    } else {
        -1
    }
}

fn get_window_size(ifd: i32, ofd: i32, rows: &mut i32, cols: &mut i32) -> i32 {
    let mut ws: raw::winsize = unsafe { mem::zeroed() };
    if unsafe { ioctl(1, raw::TIOCGWINSZ, &mut ws) } == -1 || ws.ws_col == 0 {
        let mut orig_row = 0;
        let mut orig_col = 0;
        let retval = get_cursor_position(ifd, ofd, &mut orig_row, &mut orig_col);
        if retval == -1 {
            return -1;
        }
        if unsafe { write(ofd, "\x1b[999C\x1b[999B".as_bytes().as_ptr() as *const _, 12) } != 12 {
            return -1;
        }
        let retval = get_cursor_position(ifd, ofd, rows, cols);
        if retval == -1 {
            return -1;
        }
        let seq = format!("\x1b[{};{}H", orig_row, orig_col);
        if unsafe { write(ofd, seq.as_bytes().as_ptr() as *const _, seq.len()) } == 0 {
            return -1;
        }
        return 0;
    } else {
        *cols = ws.ws_col as i32;
        *rows = ws.ws_row as i32;
        return 0;
    }
}

fn is_separator(c: u8) -> bool {
    c.is_ascii_whitespace() || b",.()+-/*=~%[];".iter().any(|&x| x == c)
}

fn editor_row_has_open_comment(row: &ERow) -> bool {
    if !row.hl.is_empty() && row.rsize > 0 && row.hl[row.rsize - 1] == HL_MLCOMMENT {
        if row.rsize < 2 || (row.render[row.rsize - 2] != '*' as u8 || row.render[row.rsize - 1] != '/' as u8) {
            return true;
        }
    }
    false
}

fn editor_update_syntax(row: &mut ERow) {
    row.hl.resize(row.rsize, HL_NORMAL);

    if let Some(syntax) = unsafe { &EDITOR.syntax } {
        let mut i = 0;
        let mut prev_sep = true;
        let mut in_string = false;
        let mut in_comment = false;
        let keywords = syntax.keywords;
        let scs = syntax.singleline_comment_start.as_bytes();
        let mcs = syntax.multiline_comment_start.as_bytes();
        let mce = syntax.multiline_comment_end.as_bytes();

        let mut p = 0;
        while p < row.render.len() {
            if prev_sep && row.render[p..].starts_with(scs) {
                row.hl[i..].fill(HL_COMMENT);
                return;
            }

            if in_comment {
                row.hl[i] = HL_MLCOMMENT;
                if row.render[p..].starts_with(mce) {
                    row.hl[i..i+2].fill(HL_MLCOMMENT);
                    i += 2;
                    p += 2;
                    in_comment = false;
                    prev_sep = true;
                    continue;
                } else {
                    p += 1;
                    i += 1;
                    continue;
                }
            } else if row.render[p..].starts_with(mcs) {
                row.hl[i..i+2].fill(HL_MLCOMMENT);
                i += 2;
                p += 2;
                in_comment = true;
                prev_sep = false;
                continue;
            }

            if in_string {
                row.hl[i] = HL_STRING;
                if row.render[p] == b'\\' && (p + 1 < row.render.len()) {
                    row.hl[i+1] = HL_STRING;
                    i += 2;
                    p += 2;
                    prev_sep = false;
                    continue;
                }
                if row.render[p] == b'"' {
                    in_string = false
                }
                i += 1;
                p += 1;
                continue;
            } else if row.render[p] == b'"' {
                in_string = true;
                row.hl[i] = HL_STRING;
                i += 1;
                p += 1;
                prev_sep = false;
                continue;
            }

            if !isprint(row.render[p] as i32) {
                row.hl[i] = HL_NONPRINT;
                i += 1;
                p += 1;
                prev_sep = false;
                continue;
            }

            if prev_sep && ((is_digit(row.render[p] as i32)) ||
                (row.render[p] == b'.' && i > 0 && row.hl[i - 1] == HL_NUMBER)) {
                row.hl[i] = HL_NUMBER;
                i += 1;
                p += 1;
                prev_sep = false;
                continue;
            }

            if prev_sep {
                for &k in keywords.iter() {
                    let kw_len = k.len();
                    let kw2 = k.ends_with('|');
                    let mut klen = kw_len;

                    if kw2 {
                        klen -= 1;
                    }

                    if row.render[p..].starts_with(&k.as_bytes()[0..klen])
                        && is_separator(row.render[p + klen])
                    {
                        row.hl[i..i+klen].fill(if kw2 { HL_KEYWORD2 } else { HL_KEYWORD1 });
                        p += klen;
                        i += klen;
                        break;
                    }
                }
                prev_sep = false;
                continue;
            }

            prev_sep = is_separator(row.render[p]);
            p += 1;
            i += 1;
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
    if unsafe { EDITOR.syntax.is_none() } {
        for syntax in HLDB.iter() {
            for ext in syntax.filematch {
                if (filename.ends_with(ext) && ext.starts_with("."))
                    || filename.contains(ext)
                {
                    unsafe {
                        EDITOR.syntax = Some(syntax.clone());
                        return;
                    }
                }
            }
        }
    }
}

fn is_digit(ch: i32) -> bool {
    ch >= b'0' as i32 && ch <= b'9' as i32
}

fn isprint(c: i32) -> bool {
    c >= 0x20 && c <= 0x7E
}

const C_HL_EXTENSIONS: [&'static str; 4] = [".c", ".h", ".cpp", ".hpp"];
const C_HL_KEYWORDS: [&'static str; 81] = [
    "auto", "break", "case", "continu  e", "default", "do", "else", "enum", "extern", "for", "goto", "if", "register", "return", "sizeof", "static", "struct",
    "switch", "typedef", "union", "volatile", "while", "NULL", "alignas", "alignof", "and", "and_eq", "asm", "bitand", "bitor", "class", "compl", "constexpr", "const_cast", "deltype", "delete",
    "dynamic_cast", "explicit", "export", "false", "friend", "inline", "mutable", "namespace", "new", "noexcept", "not", "not_eq", "nullptr", "operator", "or", "or_eq", "private", "protected", "public",
    "reinterpret_cast", "static_assert", "static_cast", "template", "this", "thread_local", "throw", "true", "try", "typeid", "typename", "virtual", "xor", "xor_eq", "int|", "long|", "double|", "float|",
    "char|", "unsigned|", "signed|", "void|", "short|", "auto|", "const|", "bool|"
];

static HLDB: [EditorSyntax; 1] = [
    EditorSyntax {
        filematch: &C_HL_EXTENSIONS,
        keywords: &C_HL_KEYWORDS,
        singleline_comment_start: "//",
        multiline_comment_start: "/*",
        multiline_comment_end: "*/",
        flags: HL_HIGHLIGHT_STRINGS | HL_HIGHLIGHT_NUMBERS,
    }
];

struct Abuf {
    b: Vec<u8>,
}

impl Abuf {
    fn new() -> Self {
        Abuf { b: Vec::new() }
    }

    fn append(&mut self, s: &str) {
        self.b.extend_from_slice(s.as_bytes());
    }

    fn free(&mut self) {
        self.b.clear();
    }
}

fn editor_refresh_screen() -> io::Result<()> {
    let mut ab = Abuf::new();

    ab.append("\x1b[?25l\x1b[H");

    for y in 0..unsafe { EDITOR.screenrows } {
        let filerow = unsafe { EDITOR.rowoff + y };
        if filerow >= unsafe { EDITOR.numrows } {
            if unsafe { EDITOR.numrows == 0 && y == EDITOR.screenrows / 3 } {
                let welcome = format!("Kilo editor -- version {}\x1b[0K\r\n", KILO_VERSION);
                let padding = (unsafe { EDITOR.screencols - welcome.len() }) / 2;
                if padding > 0 {
                    ab.append("~");
                    ab.append(&" ".repeat(padding));
                }
                ab.append(&welcome);
            } else {
                ab.append("~\x1b[0K\r\n");
            }
        } else {
            let r = &unsafe { &EDITOR.row[filerow] };
            let len = if r.rsize >= unsafe { EDITOR.coloff } {
                r.rsize - unsafe { EDITOR.coloff }
            } else {
                0
            };
            if len > 0 {
                let mut current_color: i32 = -1;
                for j in 0..len {
                    let ch = r.render[unsafe { EDITOR.coloff + j }];
                    let hl = if !r.hl.is_empty() {
                        r.hl[unsafe { EDITOR.coloff + j }]
                    } else {
                        HL_NORMAL
                    };
                    if hl == HL_NONPRINT {
                        ab.append("\x1b[7m");
                        ab.append(&format!("{}", if ch <= 26 { b'@' + ch } else { b'?' }));
                        ab.append("\x1b[0m");
                    } else if hl == HL_NORMAL {
                        if current_color != -1 {
                            ab.append("\x1b[39m");
                            current_color = -1;
                        }
                        ab.append(&format!("{}", ch as char));
                    } else {
                        let color = editor_syntax_to_color(hl);
                        if color != current_color {
                            ab.append(&format!("\x1b[{}m", color));
                            current_color = color;
                        }
                        ab.append(&format!("{}", ch as char));
                    }
                }
            }
            ab.append("\x1b[39m");
            ab.append("\x1b[0K\r\n");
        }
    }

    ab.append("\x1b[0K");
    ab.append("\x1b[7m");
    let status = format!(
        "{:.20} - {} lines {}",
        unsafe { EDITOR.filename.as_deref().unwrap_or("[No Name]") },
        unsafe { EDITOR.numrows },
        if unsafe { EDITOR.dirty } { "(modified)" } else { "" }
    );

    let rstatus = format!("{}/{}", unsafe { EDITOR.rowoff + EDITOR.cy + 1 }, unsafe { EDITOR.numrows });

    if status.len() > unsafe { EDITOR.screencols } {
        ab.append(&status[0..unsafe { EDITOR.screencols }]);
    } else {
        ab.append(&status);
        while ab.b.len() < unsafe { EDITOR.screencols } {
            if (unsafe { EDITOR.screencols } - ab.b.len()) == rstatus.len() {
                ab.append(&rstatus);
                break;
            } else {
                ab.append(" ");
            }
        }
    }
    ab.append("\x1b[0m\r\n");

    ab.append("\x1b[0K");
    let msglen = unsafe { EDITOR.statusmsg.len() };
    let statusmsg = unsafe { &EDITOR.statusmsg };
    if msglen > 0 && (SystemTime::now().duration_since(unsafe { EDITOR.statusmsg_time }).unwrap_or_else(|_| Duration::new(60, 0)).as_secs() < 5) {
        ab.append(&statusmsg[..min(msglen, unsafe { EDITOR.screencols })]);
    }

    let mut cx = 1;
    let filerow = unsafe { EDITOR.rowoff + EDITOR.cy };
    let row = if filerow >= unsafe { EDITOR.numrows } {
        None
    } else {
        Some(&unsafe { &EDITOR.row[filerow] })
    };
    if let Some(row) = row {
        for j in 0..(unsafe { EDITOR.cx + EDITOR.coloff }) {
            if j < row.size && row.chars[j] == b'\t' {
                cx += 8 - (cx % 8);
            }
            cx += 1;
        }
    }
    ab.append(&format!("\x1b[{};{}H", unsafe { EDITOR.cy + 1 }, cx));

    ab.append("\x1b[?25h");

    io::stdout().write_all(&ab.b)?;

    ab.free();
    Ok(())
}

fn editor_row_insert_char(row: &mut ERow, at: usize, c: u8) {
    if at > row.size {
        let padlen = at - row.size;
        row.chars.resize(row.size + padlen + 1, b' ');
        row.size += padlen + 1;
    } else {
        row.chars.reserve(1);
        row.chars.insert(at, c);
        row.size += 1;
    }
    editor_update_syntax(row);
    unsafe {
        EDITOR.dirty = true;
    }
}

fn editor_insert_row(at: usize, s: &str) {
    if at > unsafe { EDITOR.numrows } {
        return;
    }

    unsafe {
        EDITOR.row.insert(at, ERow::default());
        for j in at + 1..=EDITOR.numrows {
            EDITOR.row[j].idx += 1;
        }
    }

    let row = unsafe { &mut EDITOR.row[at] };
    row.size = s.len();
    row.chars = s.as_bytes().to_vec();
    row.render.clear();
    row.idx = at;
    editor_update_syntax(row);

    unsafe {
        EDITOR.numrows += 1;
        EDITOR.dirty = true;
    }
}

fn editor_update_row(row: &mut ERow) {
    let mut tabs = 0;

    for &ch in row.chars.iter() {
        if ch == KeyAction::Tab as u8 {
            tabs += 1;
        }
    }

    let allocsize = row.size + tabs * 8 + 1;
    row.render.resize(allocsize, 0);

    let mut idx = 0;
    for &ch in row.chars.iter() {
        if ch == KeyAction::Tab as u8 {
            row.render[idx] = b' ';
            idx += 1;
            while (idx + 1) % 8 != 0 {
                row.render[idx] = b' ';
                idx += 1;
            }
        } else {
            row.render[idx] = ch;
            idx += 1;
        }
    }

    row.rsize = idx;
    row.render[idx] = 0;

    editor_update_syntax(row);
}

fn editor_insert_char(c: u8) {
    let filerow = unsafe { EDITOR.rowoff + EDITOR.cy };
    let filecol = unsafe { EDITOR.coloff + EDITOR.cx };
    let row = if filerow >= unsafe { EDITOR.numrows } {
        None
    } else {
        Some(&mut unsafe { &EDITOR.row[filerow] })
    };

    if row.is_none() {
        while unsafe { EDITOR.numrows <= filerow } {
            editor_insert_row(unsafe { EDITOR.numrows }, "");
        }
    }

    let row = &mut unsafe { &mut EDITOR.row[filerow] };
    editor_row_insert_char(row, filecol, c);

    if unsafe { EDITOR.cx == EDITOR.screencols - 1 } {
        unsafe { EDITOR.coloff += 1 };
    } else {
        unsafe { EDITOR.cx += 1 };
    }
}

fn editor_open(filename: &str) -> io::Result<()> {
    unsafe {
        EDITOR.dirty = false;
        EDITOR.filename = Some(filename.to_string());

        let mut f = OpenOptions::new().read(true).open(filename)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;

        for line in s.lines() {
            editor_insert_row(EDITOR.numrows, line);
        }

        EDITOR.dirty = false;
    }

    Ok(())
}

fn editor_process_keypress(fd: i32) -> io::Result<()> {
    let c = editor_read_key(fd);
    match KeyAction::from(c) {
        KeyAction::CtrlC => {}
        KeyAction::CtrlQ => {
            if unsafe { EDITOR.dirty } {
                editor_set_status_message("WARNING!!! File has unsaved changes. Press Ctrl-Q again to quit.");
                let c = editor_read_key(fd);
                if KeyAction::CtrlQ == KeyAction::from(c) {
                    process::exit(0);
                }
            } else {
                process::exit(0);
            }
        }
        KeyAction::CtrlS => {
            // Ignored in this standalone version; usually means save.
        }
        KeyAction::CtrlF => {
            // Ignored in this standalone version; usually means find.
        }
        KeyAction::Backspace | KeyAction::CtrlH | KeyAction::DelKey => {
            // Ignored in this standalone version; usually means delete character.
        }
        _ => {
            editor_insert_char(c as u8);
        }
    }
    Ok(())
}

pub fn main() {
    if let Err(err) = std::panic::catch_unwind(|| {
        if std::env::args().len() != 2 {
            eprintln!("Usage: kilo <filename>");
            process::exit(1);
        }
    
        let filename = std::env::args().nth(1).unwrap();
        enable_raw_mode(0).unwrap();
    
        editor_open(&filename).unwrap();
        editor_select_syntax_highlight(&filename);
    
        unsafe {
            EDITOR.rawmode = true;
        }
    
        editor_set_status_message("HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find");
    
        while {
            editor_refresh_screen().unwrap();
            editor_process_keypress(0).unwrap();
            true
        } {}
    
    }) {
        eprintln!("Unhandled panic: {:?}", err);
    }
}