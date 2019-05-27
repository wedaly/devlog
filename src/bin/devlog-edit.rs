extern crate chrono;
extern crate devlog;

use chrono::Local;
use devlog::config::Config;
use devlog::locator::Locator;

fn main() -> Result<(), Error> {
    let config = Config::load();
    let today = Local::today();
    match Locator::OnDate(today).path(&config) {
        Some(p) => println!("today = {:?}", p),
        None => println!("could not build path today"),
    }

    match Locator::MostRecentBeforeDate(today).path(&config) {
        Some(p) => println!("yesterday = {:?}", p),
        None => println!("could not build path yesterday"),
    }
    Ok(())
}

#[derive(Debug)]
enum Error {}
