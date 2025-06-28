
use ratatui::prelude::*;
use ratatui::widgets::{Borders, Clear};
use ratatui::{
    Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph, List, ListDirection},
};

use crate::{App, CurrentEditMode, CurrentScreen};

pub fn ui(frame: &mut Frame, app: &mut App)
{
    frame.render_widget(Clear, frame.area());
    match app.current_screen
    {
        CurrentScreen::Main => {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(frame.area());
                
                        // Left panel
            let left_title = Line::from("Options").bold().blue().centered();
            let list = List::new(app.choices.clone())
                .style(Style::new().white())
                .highlight_style(Style::new().yellow())
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom)
                .block(Block::bordered().title(left_title));

            frame.render_widget(list, layout[0]);

            // Right panel
            let title = Line::from("Ramdis")
                .bold()
                .blue()
                .centered();

            let text =
                "This is not something related to ram-disk.\n\
                This. Is. Distro. Hopping. Killer.\n\n\n\
                Press `Ctrl-C` or `q` to quit.\n\
                Press `n` to create a new choice.\n\
                Press `e` to edit a choice.\n\n\
                While in editing/modifying mode:\n\
                `Ctrl` + `i` to change the item's index;\n\
                `Ctrl` + `d` to change the choice's data.\n\n\
                All shortcuts are in-case-sensitive.\n\
                The list is zero-indexed.";

            frame.render_widget(
                Paragraph::new(text)
                    .block(Block::bordered().title(title))
                    .centered(),
                layout[1]
            )
        }
        CurrentScreen::Exiting => {
            let popup_block = Block::bordered()
                .title("Question")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Red));

            let exit_text = Text::styled(
                "Are you sure want to exit the application?\n\
                Press Ctrl-C` or `q` to confirm. `ESC` to cancel.",
                Style::default().fg(Color::White)
            );
        
            frame.render_widget(Paragraph::new(exit_text).block(popup_block), centered_rect(60, 25, frame.area()));
        },
        CurrentScreen::Editing | CurrentScreen::Creating => {
            let mut popup_block = Block::bordered()
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray));

            let area = centered_rect(60, 25, frame.area());

            let popup_halfs = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            let mut str_block = Block::default().title("Choice").borders(Borders::ALL);
            let mut idx_block = Block::default().title("Index").borders(Borders::ALL);

            let active_style = Style::default().bg(Color::LightMagenta).fg(Color::Black);

            match app.current_emode
            {
                CurrentEditMode::Data => {
                    frame.set_cursor_position(Position::new(
                        popup_halfs[0].x + app.current_curpos + 1,
                        popup_halfs[0].y + 1
                    ));
                    str_block = str_block.style(active_style)
                },
                CurrentEditMode::Index => {
                    frame.set_cursor_position(Position::new(
                        popup_halfs[1].x + app.current_curpos + 1,
                        popup_halfs[1].y + 1
                    ));
                    idx_block = idx_block.style(active_style)
                },
                CurrentEditMode::None => {}
            }

            popup_block = popup_block.title(
                if let CurrentScreen::Creating = app.current_screen { "New choice" }
                else { "Modify an existing choice" }
            );

            let str_text = Paragraph::new(app.current_input.clone()).block(str_block);
            frame.render_widget(str_text, popup_halfs[0]);

            let idx_text = Paragraph::new(app.current_index.to_string()).block(idx_block);
            frame.render_widget(idx_text, popup_halfs[1]);

            frame.render_widget(popup_block.clone(), area);
        },
    } 
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect
{
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}