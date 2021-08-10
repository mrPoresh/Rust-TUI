#![allow(non_snake_case)]

use chrono::prelude::*;     //A convenience module appropriate for glob imports
use crossterm::{    
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Deserialize, Serialize};    //For serializing and deserializing
use std::fs;    //Basic methods to manipulate the contents of the local filesystem
use std::io;    //Input/Output
use std::sync::mpsc;    //This module provides message-based communication over channels
use std::thread;
use std::time::{Duration, Instant};   //TIME
use thiserror::Error;
use tui::{      //Framework for terminal
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};

const DB_PATH: &str = "./data/db.json";

#[derive(Error, Debug)]     //some implementing error handling
pub enum Error {

    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),

}

#[derive(Serialize, Deserialize, Clone)]
struct Car {
    
    id: usize,
    name: String,
    model: String,
    engine: String,
    category: String,
    age: usize,
    created_at: DateTime<Utc>,

}

enum Event<I> {     //Data structure for input events

    Input(I),
    Tick,

}

enum MenuItem {

    Home,
    Cars,

}

impl From<MenuItem> for usize {             //This enables us to use the enum within 
    fn from(input: MenuItem) -> usize {     // the Tabs component of TUI to highlight the 
        match input {                       // currently selected tab in the menu

            MenuItem::Home => 0,
            MenuItem::Cars => 1,

        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

//****************************************************************** Setup to create an input loop ******************************************************************

    enable_raw_mode().expect("\nCan run in raw mode\n");    //which eliminates the need to wait for an Enter by the user to react to the input
    
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {

        let mut last_tick = Instant::now();

        loop {      //input loop

            //This logic is spawned in another thread because we need our main thread to render the application.
            //This way, our input loop doesn’t block the rendering

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }

        }

    });

    let stdout = io::stdout();                      //We defined a CrosstermBackend using stdout and used it in a TUI Terminal,
    let backend = CrosstermBackend::new(stdout);    // clearing it initially and implicitly checking that everything works.
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

//******************************************************************************************************************************************************************

}