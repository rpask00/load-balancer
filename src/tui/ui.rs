use crate::tui::app::App;
use std::sync::Arc;

use crate::load_balancer::worker::WorkerStatus;
use ratatui::text::{Line, Span};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    render_header(f, chunks[0], app);
    render_table(f, chunks[1], app);

    if app.add_item_menu.is_some() {
        render_add_popup(f, area, app);
    }
    if app.options_menu.is_some() {
        render_options_menu(f, app);
    }
    if app.mode_selector_menu.is_some() {
        render_mode_select_popup(f, app);
    }
}

fn render_header(f: &mut Frame, area: Rect, app: &mut App) {
    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Fill(1),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(13),
        ])
        .split(area);

    app.main_menu.add_button_area = Some(header_layout[2]);
    app.main_menu.delete_button_area = Some(header_layout[3]);
    app.main_menu.options_button_area = Some(header_layout[4]);

    let current_mode = match app.load_balancer.read() {
        Ok(lb) => lb.strategy.policy().to_string(),
        Err(_) => String::from(""),
    };

    let title_text = format!("Load Balancer | {}", &current_mode);
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).bold())
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, header_layout[0]);

    let add_btn = Paragraph::new("[ Add ]")
        .style(Style::default().fg(Color::Green).bold())
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(add_btn, header_layout[2]);

    let del_btn = Paragraph::new("[ Del ]")
        .style(Style::default().fg(Color::Green).bold())
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(del_btn, header_layout[3]);

    let options_btn = Paragraph::new("[ Options ]")
        .style(Style::default().fg(Color::Yellow).bold())
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(options_btn, header_layout[4]);
}

fn connection_bar(connections: usize, capacity: u8) -> Cell<'static> {
    const BAR_WIDTH: usize = 90;
    let ratio = if capacity == 0 {
        0.0
    } else {
        (connections as f64 / capacity as f64).min(1.0)
    };
    let filled = (ratio * BAR_WIDTH as f64).round() as usize;

    let spans: Vec<Span> = (0..BAR_WIDTH)
        .map(|i| {
            if i < filled {
                Span::styled("█", Style::default().fg(Color::Green).dim())
            } else {
                Span::styled("░", Style::default().fg(Color::Red).dim())
            }
        })
        .collect();

    Cell::from(Line::from(spans))
}

fn render_table(f: &mut Frame, area: Rect, app: &mut App) {
    app.main_menu.table_area = Some(area);

    let header = Row::new(["Port", "Threads", "Status", "Load"])
        .style(Style::default().fg(Color::Yellow).bold())
        .bottom_margin(1)
        .top_margin(1);

    let workers = &app
        .load_balancer
        .read()
        .expect("Failed to lock load balancer for reading")
        .workers;

    let rows: Vec<Row> = workers
        .iter()
        .map(|worker| {
            let status = match worker.status.read() {
                Ok(status) => *status,
                Err(_) => WorkerStatus::Unknown,
            };

            Row::new(vec![
                Cell::from(worker.port.to_string()),
                Cell::from(worker.num_threads.to_string()),
                Cell::from(status.to_string()),
                connection_bar(Arc::strong_count(worker) - 1, worker.num_threads),
                Cell::from(Arc::strong_count(worker).to_string()),
            ])
            .height(1)
            .bottom_margin(0)
        })
        .collect();

    let widths = [
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(65),
        Constraint::Percentage(5),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Workers"))
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_add_popup(f: &mut Frame, area: Rect, app: &mut App) {
    if let Some(menu) = &mut app.add_item_menu {
        let popup_area = centered_rect(60, 40, area);
        menu.popup_area = Some(popup_area);

        let popup = Block::default()
            .title(" Add New Worker ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        f.render_widget(Clear, popup_area);
        f.render_widget(popup, popup_area);

        let inner = popup_area.inner(Margin::new(1, 1));
        let input_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(inner);

        menu.name_input_area = Some(input_layout[1]);
        menu.port_input_area = Some(input_layout[4]);

        let name_style = if menu.focused == crate::tui::models::InputField::Name {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default()
        };
        let name_label = Paragraph::new("Name:").style(name_style);
        let name_input = Paragraph::new(menu.name.as_str())
            .block(Block::default().borders(Borders::ALL))
            .style(name_style);
        f.render_widget(name_label, input_layout[0]);
        f.render_widget(name_input, input_layout[1]);

        let port_style = if menu.port_error {
            Style::default().fg(Color::Red).bold()
        } else if menu.focused == crate::tui::models::InputField::Port {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default()
        };
        let port_label = Paragraph::new("Port:").style(port_style);
        let port_input = Paragraph::new(menu.port_str.as_str())
            .block(Block::default().borders(Borders::ALL))
            .style(port_style);
        f.render_widget(port_label, input_layout[3]);
        f.render_widget(port_input, input_layout[4]);
    }
}

fn render_options_menu(f: &mut Frame, app: &mut App) {
    if let Some(stored_area) = &mut app.options_menu {
        if let Some(btn_area) = app.main_menu.options_button_area {
            let full_area = f.area();
            let menu_width: u16 = 22;
            let menu_height: u16 = 5;

            let x = btn_area
                .right()
                .saturating_sub(menu_width)
                .min(full_area.right().saturating_sub(menu_width));

            let menu_area = Rect::new(x, btn_area.bottom(), menu_width, menu_height);

            *stored_area = menu_area;

            f.render_widget(Clear, menu_area);

            let block = Block::default()
                .title(" Options ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White));
            f.render_widget(block, menu_area);

            let inner = menu_area.inner(Margin::new(1, 1));
            let item_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1)])
                .split(inner);

            let set_mode_item = Paragraph::new(" Set Mode ").style(Style::default());
            let quit_item = Paragraph::new(" Quit ").style(Style::default().fg(Color::Red).bold());

            f.render_widget(set_mode_item, item_layout[0]);
            f.render_widget(quit_item, item_layout[1]);
        }
    }
}

fn render_mode_select_popup(f: &mut Frame, app: &mut App) {
    if let Some(menu) = &mut app.mode_selector_menu {
        let popup_area = centered_rect(55, 50, f.area());
        menu.menu_area = Some(popup_area);

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(" Select Load Balancer Mode ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(block, popup_area);

        let inner = popup_area.inner(Margin::new(2, 2));
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(inner);

        let current_text = format!("Mode: {}", &app.current_mode);
        let current = Paragraph::new(current_text).style(Style::default().fg(Color::Green).bold());
        f.render_widget(current, layout[0]);

        let rr_style = if menu.selection_index == 0 {
            Style::default()
                .fg(Color::Yellow)
                .bold()
                .add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let lc_style = if menu.selection_index == 1 {
            Style::default()
                .fg(Color::Yellow)
                .bold()
                .add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        let ll_style = if menu.selection_index == 2 {
            Style::default()
                .fg(Color::Yellow)
                .bold()
                .add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        let rr = Paragraph::new(if menu.selection_index == 0 {
            "▶ Round Robin"
        } else {
            "  Round Robin"
        })
        .style(rr_style);
        let lc = Paragraph::new(if menu.selection_index == 1 {
            "▶ Least Connections"
        } else {
            "  Least Connections"
        })
        .style(lc_style);
        let ll = Paragraph::new(if menu.selection_index == 2 {
            "▶ Least Load"
        } else {
            "  Least Load"
        })
        .style(ll_style);

        f.render_widget(rr, layout[2]);
        f.render_widget(lc, layout[3]);
        f.render_widget(ll, layout[4]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
