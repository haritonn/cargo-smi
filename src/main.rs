mod parser;

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use parser::get_info;
use std::{
    io::{Write, stdout},
    thread,
    time::Duration,
};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();

    loop {
        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        let res = get_info()?;
        println!("{:?}", res);
        stdout.flush()?;
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}
