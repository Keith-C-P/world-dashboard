use chrono::Local;
use chrono_tz::Tz;
use crossterm::terminal;
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::thread;
use std::time::Duration;
use strip_ansi_escapes::strip;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use serde::Deserialize;
use std::env;
use std::fs;


const CARD_WIDTH: usize = 36;
const CARD_GAP: usize = 1;

#[derive(Debug, Deserialize)]
struct Person {
    name: String,
    location: String,
    timezone: String,
}


// const PEOPLE: &[Person] = &[
//     Person {
//         name: "YOU",
//         location: "Hyderabad",
//         timezone: "Asia/Kolkata",
//     },
//     Person {
//         name: "ALICE",
//         location: "London",
//         timezone: "Europe/London",
//     },
//     Person {
//         name: "BOB",
//         location: "New York",
//         timezone: "America/New_York",
//     },
//     Person {
//         name: "CHARLIE",
//         location: "Tokyo",
//         timezone: "Asia/Tokyo",
//     },
//     Person {
//         name: "CHARLIE",
//         location: "Tokyo",
//         timezone: "Asia/Tokyo",
//     },
//     Person {
//         name: "CHARLIE",
//         location: "Tokyo",
//         timezone: "Asia/Tokyo",
//     },
//     ];


fn load_people() -> Vec<Person> {
let path = env::var("WORLD_DASHBOARD_CONFIG")
    .unwrap_or_else(|_| {
        "config/people.json".to_string()
    });

    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|error| {
            panic!(
                "Failed to read '{}': {}",
                path,
                error,
            );
        });

    serde_json::from_str(&contents)
        .unwrap_or_else(|error| {
            panic!(
                "Failed to parse '{}': {}",
                path,
                error,
            );
        })
}


fn visible_width(text: &str) -> usize {
    let stripped = strip(text.as_bytes());

    let text = String::from_utf8_lossy(&stripped);

    text.width()
}


fn truncate_to_width(
    text: &str,
    width: usize,
) -> String {
    let stripped = strip(text.as_bytes());

    let text = String::from_utf8_lossy(&stripped);

    let mut current_width = 0;
    let mut result = String::new();

    for character in text.chars() {
        let character_width =
            UnicodeWidthChar::width(character)
                .unwrap_or(0);

        if current_width + character_width > width {
            break;
        }

        result.push(character);
        current_width += character_width;
    }

    result
}


fn get_weather(
    client: &Client,
    location: &str,
) -> String {
    let encoded_location =
        urlencoding::encode(location);

    let url = format!(
        "https://wttr.in/{}?0",
        encoded_location,
    );

    for _attempt in 0..3 {
        let result = client
            .get(&url)
            .header(
                "User-Agent",
                "curl",
            )
            .timeout(
                Duration::from_secs(15),
            )
            .send();

        match result {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text() {
                        Ok(text) => {
                            return text
                                .trim_end()
                                .to_string();
                            }

                        Err(_) => {}
                    }
                }
            }

            Err(_) => {}
        }

        thread::sleep(
            Duration::from_millis(250),
        );
    }

    "Weather unavailable".to_string()
}


fn get_local_time(
    timezone: &str,
) -> String {
    let timezone: Tz = timezone
        .parse()
        .expect("Invalid timezone");

    let now = Local::now()
        .with_timezone(&timezone);

    now.format(
        "%H:%M · %a %b %d"
    )
        .to_string()
}


fn render_person(
    client: &Client,
    person: &Person,
) -> Vec<String> {
    let weather = get_weather(
        client,
        &person.location,
    );

    let local_time = get_local_time(
        &person.timezone,
    );

    let mut lines = vec![
        person.name.clone(),
        person.location.clone(),
        local_time,
        String::new(),
    ];

    lines.extend(
        weather
            .lines()
            .map(|line| {
                line.trim_end()
                    .to_string()
            }),
    );

    lines
}


fn format_line(
    line: &str,
) -> String {
    let content_width =
        CARD_WIDTH - 4;

    let line = line.trim_end();

    let line_width = visible_width(line);

    let line = if line_width > content_width {
        truncate_to_width(
            line,
            content_width,
        )
    } else {
        line.to_string()
    };

    let line_width = visible_width(&line);

    let padding =
        content_width - line_width;

    format!(
        "│ {}{} │",
        line,
        " ".repeat(padding),
    )
}


fn make_card(
    lines: &[String],
) -> Vec<String> {
    let border_width =
        CARD_WIDTH - 2;

    let top = format!(
        "╭{}╮",
        "─".repeat(border_width),
    );

    let bottom = format!(
        "╰{}╯",
        "─".repeat(border_width),
    );

    let mut card = Vec::new();

    card.push(top);

    card.extend(
        lines.iter().map(|line| {
            format_line(line)
        }),
    );

    card.push(bottom);

    card
}


fn print_world_header(
    width: usize,
) {
    let title =
        "WORLD TIME DASHBOARD";

    let title_width =
        visible_width(title);

    if width < title_width + 2 {
        println!("{}", title);
        println!();
        return;
    }

    let inner_width =
        width - 2;

    let left_padding =
        (inner_width - title_width)
        / 2;

    let right_padding =
        inner_width
        - title_width
        - left_padding;

    println!(
        "╭{}╮",
        "─".repeat(inner_width),
    );

    println!(
        "│{}{}{}│",
        " ".repeat(left_padding),
        title,
        " ".repeat(right_padding),
    );

    println!(
        "╰{}╯",
        "─".repeat(inner_width),
    );
}


fn get_cards_per_row(
    card_count: usize,
    terminal_width: usize,
) -> usize {
    let total_width_per_card =
        CARD_WIDTH + CARD_GAP;

    std::cmp::min(
        card_count,
        std::cmp::max(
            1,
            (
                terminal_width
                + CARD_GAP
            ) / total_width_per_card,
        ),
    )
}


fn get_layout_width(
    cards_per_row: usize,
) -> usize {
    cards_per_row * CARD_WIDTH
        + (
            cards_per_row - 1
        ) * CARD_GAP
}


fn print_cards(
    cards: &mut [Vec<String>],
    terminal_width: usize,
) {
    let cards_per_row =
        get_cards_per_row(
            cards.len(),
            terminal_width,
        );

    for start in (0..cards.len())
        .step_by(cards_per_row)
        {
            let end = std::cmp::min(
                start + cards_per_row,
                cards.len(),
            );

            let row_cards =
                &mut cards[start..end];

            let max_height =
                row_cards
                .iter()
                .map(|card| card.len())
                .max()
                .unwrap_or(0);

            for card in row_cards.iter_mut() {
                while card.len() < max_height {
                    card.push(
                        format_line(""),
                    );
                }
            }

            for row in 0..max_height {
                let line = row_cards
                    .iter()
                    .map(|card| {
                        card[row].as_str()
                    })
                .collect::<Vec<_>>()
                    .join(
                        &" ".repeat(CARD_GAP),
                    );

                println!("{}", line);
            }

            if end < cards.len() {
                println!();
            }
        }
}


fn main() {
    let terminal_width =
        terminal::size()
        .map(|(width, _height)| {
            width as usize
        })
        .unwrap_or(80);

    let people = load_people();

    let client = Client::builder()
        .connect_timeout(
            Duration::from_secs(5),
        )
        .build()
        .expect(
            "Failed to create HTTP client",
        );

    let people_cards: Vec<Vec<String>> =
        people
        .par_iter()
        .map(|person| {
            render_person(
                &client,
                person,
            )
        })
        .collect();

    let mut cards: Vec<Vec<String>> =
        people_cards
        .iter()
        .map(|person| {
            make_card(person)
        })
        .collect();

    let cards_per_row =
        get_cards_per_row(
            cards.len(),
            terminal_width,
        );

    let layout_width =
        get_layout_width(
            cards_per_row,
        );

    print_world_header(
        layout_width,
    );

    print_cards(
        &mut cards,
        terminal_width,
    );
}
