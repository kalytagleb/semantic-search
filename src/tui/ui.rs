use crate::tui::{Screen, TuiApp};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph},
};

pub fn render(f: &mut Frame, app: &mut TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(f.size());

    match app.current_screen {
        Screen::Indexing => {
            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("INDEXING PROGRESS")
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Magenta)),
                )
                .gauge_style(Style::default().fg(Color::Magenta).bg(Color::Black))
                .percent((app.progress * 100.0) as u16);

            f.render_widget(gauge, chunks[1]);
        }

        Screen::Searching => {
            let input = Paragraph::new(app.query.as_str()).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("ENTER QUERY")
                    .border_style(Style::default().fg(Color::Cyan)),
            );

            f.render_widget(input, chunks[0]);

            let items: Vec<ListItem> = app
                .results
                .iter()
                .map(|res| {
                    ListItem::new(format!(" {} (Score: {:.2})", res.path, res.score))
                        .style(Style::default().fg(Color::Yellow))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("SEARCH RESULTS  [↑↓ to scroll]")
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            f.render_stateful_widget(list, chunks[1], &mut app.list_state);
        }
    }

    let status = Paragraph::new(app.status_text.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_type(BorderType::Double),
        );

    f.render_widget(status, chunks[2])
}
