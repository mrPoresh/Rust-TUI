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
    engine: usize,
    category: String,
    age: usize,
    created_at: DateTime<Utc>,

}

enum Event<I> {     //Data structure for input events

    Input(I),
    Tick,

}

#[derive(Copy, Clone, Debug)]
enum MenuItem {

    Home,
    Cars,
    Joke,

}

impl From<MenuItem> for usize {             //This enables us to use the enum within 
    fn from(input: MenuItem) -> usize {     // the Tabs component of TUI to highlight the 
        match input {                       // currently selected tab in the menu

            MenuItem::Home => 0,
            MenuItem::Cars => 1,
            MenuItem::Joke => 2,

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

 //************************************************************** Loop for rendering widgets ************************************************************************

    let menu_titles = vec!["Home", "Cars", "Add", "Delete", "Quit"];
    let mut active_menu_item = MenuItem::Home;

    let mut car_list_state = ListState::default();
    car_list_state.select(Some(0));

    loop {

        terminal.draw(|rect| {

            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let copyright = Paragraph::new("Car-CLI 2021 - Protected by God")    //Static footer with the fake copyright
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),
                );

            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));
            rect.render_widget(tabs, chunks[0]);

            match active_menu_item {

                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::Joke => rect.render_widget(render_joke(), chunks[1]),
                MenuItem::Cars => {

                    let cars_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_cars(&car_list_state);
                    rect.render_stateful_widget(left, cars_chunks[0], &mut car_list_state);
                    rect.render_widget(right, cars_chunks[1]);

                }

            }

            rect.render_widget(copyright, chunks[2]);

        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {   //binding keys

                KeyCode::Char('q') => {

                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;

                }

                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('c') => active_menu_item = MenuItem::Cars,
                KeyCode::Char('a') => {

                    add_random_car_to_db().expect("can add new random car");
                }

                KeyCode::Char('d') => {

                    remove_car_at_index(&mut car_list_state).expect("can remove car");
                
                }

                KeyCode::Char('j') => active_menu_item = MenuItem::Joke,

                KeyCode::Down => {

                    if let Some(selected) = car_list_state.selected() {

                        let amount_cars = read_db().expect("can fetch car list").len();
                        if selected >= amount_cars - 1 {

                            car_list_state.select(Some(0));

                        } else {

                            car_list_state.select(Some(selected + 1));

                        }

                    }

                }

                KeyCode::Up => {

                    if let Some(selected) = car_list_state.selected() {

                        let amount_cars = read_db().expect("can fetch car list").len();
                        if selected > 0 {

                            car_list_state.select(Some(selected - 1));

                        } else {

                            car_list_state.select(Some(amount_cars - 1));

                        }

                    }

                }

                _ => {}

            },

            Event::Tick => {}

        }

    }

    Ok(())

}

fn render_home<'a>() -> Paragraph<'a> {

    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Car-CLI",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'c' to access cars, 'a' to add random new car and 'd' to delete the currently selected car and 'j' for the best joke")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );

    home

}

fn render_joke<'a>() -> Paragraph<'a> {

    let joke = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "JOKE MINUTE",
            Style::default().fg(Color::LightRed),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Knok! Knok!")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Who's there?")]),
        Spans::from(vec![Span::raw("Control Freak.")]),
        Spans::from(vec![Span::raw("Con...")]),
        Spans::from(vec![Span::styled(
            "OK, now you say, 'Control Freak who?'",
            Style::default().fg(Color::Red),
        )]),

    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Joke")
            .border_type(BorderType::Plain),
    );

    joke

}

fn render_cars<'a>(car_list_state: &ListState) -> (List<'a>, Table<'a>) {
    
    let cars = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Cars")
        .border_type(BorderType::Plain);

    let car_list = read_db().expect("can fetch car list");

    let items: Vec<_> = car_list
        .iter()
        .map(|car| {
            ListItem::new(Spans::from(vec![Span::styled(
                car.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_car = car_list
        .get(
            car_list_state
                .selected()
                .expect("there is always a selected car"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(cars).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let car_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_car.id.to_string())),
        Cell::from(Span::raw(selected_car.name)),
        Cell::from(Span::raw(selected_car.model)),
        Cell::from(Span::raw(selected_car.engine.to_string())),
        Cell::from(Span::raw(selected_car.category)),
        Cell::from(Span::raw(selected_car.age.to_string())),
        Cell::from(Span::raw(selected_car.created_at.to_string())),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Model",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Engine",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Category",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Age",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created At",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(5),
        Constraint::Percentage(15),
    ]);

    (list, car_detail)
}

fn read_db() -> Result<Vec<Car>, Error> {

    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Car> = serde_json::from_str(&db_content)?;

    Ok(parsed)

}

fn add_random_car_to_db() -> Result<Vec<Car>, Error> {

    let mut rng = rand::thread_rng();
    let db_content = fs::read_to_string(DB_PATH)?;
    let mut parsed: Vec<Car> = serde_json::from_str(&db_content)?;
    let car_type = match rng.gen_range(0, 1) {

        0 => "coupe",
        _ => "sedan",

    };

    let random_car = Car {

        id: rng.gen_range(0, 100),
        name: rng.sample_iter(Alphanumeric).take(10).collect(),
        model: rng.sample_iter(Alphanumeric).take(10).collect(),
        engine: rng.gen_range(0, 5),
        category: car_type.to_owned(),
        age: rng.gen_range(1, 30),
        created_at: Utc::now(),

    };

    parsed.push(random_car);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;

    Ok(parsed)

}

fn remove_car_at_index(car_list_state: &mut ListState) -> Result<(), Error> {

    if let Some(selected) = car_list_state.selected() {

        let db_content = fs::read_to_string(DB_PATH)?;
        let mut parsed: Vec<Car> = serde_json::from_str(&db_content)?;
        parsed.remove(selected);
        fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
        car_list_state.select(Some(selected - 1));

    }

    Ok(())

}