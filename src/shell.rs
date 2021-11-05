use core::sync::atomic::{AtomicU16, Ordering};

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use hashbrown::HashMap;
use lazy_static::lazy_static;
use pc_keyboard::DecodedKey;
use spin::Mutex;

use crate::{
    interrupts::register_keyboard_handler, print, println, writer::init_cursor, writer::WRITER,
};
struct CommandErr<'a>(&'a str);
fn load_programs() {
    register_command(
        "test",
        |args| {
            println!("test");
            if args[0] == "err" {
                Err(CommandErr("Test program err"))
            } else {
                Ok(())
            }
        },
        "test program",
    )
}
pub fn init_shell() {
    init_cursor();
    register_keyboard_handler(keyboard_handler);
    load_programs();
    print!("> ");
}
lazy_static! {
    static ref SHELL_INPUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());
}
static SHELL_CURSOR: AtomicU16 = AtomicU16::new(0);
fn keyboard_handler(key: DecodedKey) {
    match key {
        DecodedKey::Unicode(character) => match character as u8 {
            0x20..=0x7e => {
                SHELL_CURSOR.fetch_add(1, Ordering::Relaxed);
                SHELL_INPUT.lock().push(character as u8);
                print!("{}", character);
            }
            b'\n' => {
                print!("\n");
                exec(String::from_utf8(SHELL_INPUT.lock().to_vec()).unwrap());
                print!("> ");
                SHELL_CURSOR.store(1, Ordering::Relaxed);
                SHELL_INPUT.lock().clear();
            }
            0x08 => {
                if SHELL_CURSOR.load(Ordering::Relaxed) > 0 {
                    SHELL_CURSOR.fetch_sub(1, Ordering::Relaxed);
                    WRITER.lock().delete_last();
                    SHELL_INPUT.lock().pop();
                }
            }
            _ => panic!("unexpected character {:?}", character),
        },
        DecodedKey::RawKey(_key) => {
            // todo: handle arrow keys
        }
    }
}
lazy_static! {
    static ref COMMANDS: Mutex<HashMap<String, fn(Vec<&str>) -> Result<(), CommandErr>>> =
        Mutex::new(HashMap::new());
    static ref COMMANDS_DESC: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}
fn register_command(command: &str, f: fn(Vec<&str>) -> Result<(), CommandErr>, desc: &str) {
    COMMANDS.lock().insert(command.to_string(), f);
    COMMANDS_DESC
        .lock()
        .insert(command.to_string(), desc.to_string());
}
fn exec(input: String) {
    let split = input.split_whitespace().collect::<Vec<&str>>();
    let command = split[0];
    let args = split.iter().skip(1).map(|s| *s).collect::<Vec<&str>>();

    if command == "help" {
        println!("help - print this help message");
        for (command, desc) in COMMANDS_DESC.lock().iter() {
            println!("{} - {}", command, desc);
        }
    } else if COMMANDS.lock().contains_key(command) {
        COMMANDS.lock().get(command).unwrap()(args).unwrap_or_else(|e| {
            println!("err: {}", e.0);
        });
    } else {
        println!("unknown command `{}`", command)
    }
}
