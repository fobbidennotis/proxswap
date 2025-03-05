use crate::configuration::{Configuration, IptablesRule, Proxy};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Clear},
    style::Color,
};
use std::{error::Error, io, process::Command};
use crate::bindings;

pub enum InputMode {
    Normal,
    Editing,
    Creating,
}

pub enum Focus {
    ConfigList,
    ProxyList,
    RulesList,
}

pub enum CreationField {
    Name,
    ProxyType,
    ProxyUrl,
    ProxyPort,
    RedirectPorts,
    Confirm,
}

pub struct CreationState {
    current_field: CreationField,
    name: String,
    proxy_type: String,
    proxy_url: String,
    proxy_port: String,
    redirect_ports: Vec<String>,  
    current_port_input: String,   
}

impl CreationState {
    fn new() -> Self {
        CreationState {
            current_field: CreationField::Name,
            name: String::new(),
            proxy_type: String::new(),
            proxy_url: String::new(),
            proxy_port: String::new(),
            redirect_ports: Vec::new(),
            current_port_input: String::new(),
        }
    }

    fn next_field(&mut self) {
        self.current_field = match self.current_field {
            CreationField::Name => CreationField::ProxyType,
            CreationField::ProxyType => CreationField::ProxyUrl,
            CreationField::ProxyUrl => CreationField::ProxyPort,
            CreationField::ProxyPort => CreationField::RedirectPorts,
            CreationField::RedirectPorts => CreationField::Confirm,
            CreationField::Confirm => CreationField::Confirm,
        };
    }

    fn previous_field(&mut self) {
        self.current_field = match self.current_field {
            CreationField::Name => CreationField::Name,
            CreationField::ProxyType => CreationField::Name,
            CreationField::ProxyUrl => CreationField::ProxyType,
            CreationField::ProxyPort => CreationField::ProxyUrl,
            CreationField::RedirectPorts => CreationField::ProxyPort,
            CreationField::Confirm => CreationField::RedirectPorts,
        };
    }
}

pub struct App {
    configurations: Vec<Configuration>,
    active_config_index: Option<usize>,
    config_list_state: ListState,
    input_mode: InputMode,
    focus: Focus,
    search_query: String,
    filtered_configs: Vec<usize>, 
    creation_state: Option<CreationState>,
}

impl App {
    pub fn new(configurations: Vec<Configuration>) -> Self {
        let filtered_configs: Vec<usize> = (0..configurations.len()).collect();
        App {
            configurations,
            active_config_index: None,
            config_list_state: ListState::default(),
            input_mode: InputMode::Normal,
            focus: Focus::ConfigList,
            search_query: String::new(),
            filtered_configs,
            creation_state: None,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.ensure_sudo_access().await?;

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let _res = self.run_app(&mut terminal).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    async fn ensure_sudo_access(&self) -> Result<(), Box<dyn Error>> {
        let status = Command::new("sudo")
            .arg("-v")
            .status()?;

        if !status.success() {
            return Err("Failed to obtain sudo privileges".into());
        }
        Ok(())
    }

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('e') => self.input_mode = InputMode::Editing,
                            KeyCode::Char('c') => {
                                self.input_mode = InputMode::Creating;
                                self.creation_state = Some(CreationState::new());
                            }
                            KeyCode::Char('x') => {
                                self.deactivate_proxy().await;
                            }
                            KeyCode::Char('/') => {
                                self.search_query.clear();
                                self.input_mode = InputMode::Editing;
                            }
                            KeyCode::Down => self.next(),
                            KeyCode::Up => self.previous(),
                            KeyCode::Enter => {
                                if let Some(index) = self.config_list_state.selected() {
                                    if let Some(real_index) = self.filtered_configs.get(index) {
                                        self.active_config_index = Some(*real_index);
                                        self.configurations[*real_index].run().await;
                                    }
                                }
                            }
                            KeyCode::Delete => self.delete_selected(),
                            KeyCode::Tab => self.cycle_focus(),
                            _ => {}
                        }
                    }
                    InputMode::Editing => {
                        match key.code {
                            KeyCode::Enter => {
                                self.input_mode = InputMode::Normal;
                                self.filter_configurations();
                            }
                            KeyCode::Char(c) => {
                                self.search_query.push(c);
                                self.filter_configurations();
                            }
                            KeyCode::Backspace => {
                                self.search_query.pop();
                                self.filter_configurations();
                            }
                            KeyCode::Esc => {
                                self.input_mode = InputMode::Normal;
                                self.search_query.clear();
                                self.filter_configurations();
                            }
                            _ => {}
                        }
                    }
                    InputMode::Creating => {
                        match key.code {
                            KeyCode::Esc => {
                                self.input_mode = InputMode::Normal;
                                self.creation_state = None;
                            }
                            KeyCode::Down => {
                                if let Some(creation_state) = &mut self.creation_state {
                                    creation_state.next_field();
                                }
                            }
                            KeyCode::Up => {
                                if let Some(creation_state) = &mut self.creation_state {
                                    creation_state.previous_field();
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(creation_state) = &mut self.creation_state {
                                    match creation_state.current_field {
                                        CreationField::RedirectPorts => {
                                            if !creation_state.current_port_input.is_empty() {
                                                creation_state.redirect_ports.push(
                                                    creation_state.current_port_input.clone()
                                                );
                                                creation_state.current_port_input.clear();
                                            }
                                        }
                                        CreationField::Confirm => {
                                            self.create_configuration().await;
                                            self.input_mode = InputMode::Normal;
                                            self.creation_state = None;
                                        }
                                        _ => creation_state.next_field(),
                                    }
                                }
                            }
                            KeyCode::Char(c) => {
                                if let Some(creation_state) = &mut self.creation_state {
                                    match creation_state.current_field {
                                        CreationField::Name => creation_state.name.push(c),
                                        CreationField::ProxyType => creation_state.proxy_type.push(c),
                                        CreationField::ProxyUrl => creation_state.proxy_url.push(c),
                                        CreationField::ProxyPort => {
                                            if c.is_ascii_digit() {
                                                creation_state.proxy_port.push(c);
                                            }
                                        }
                                        CreationField::RedirectPorts => {
                                            if c.is_ascii_digit() {
                                                creation_state.current_port_input.push(c);
                                            }
                                        }
                                        CreationField::Confirm => {}
                                    }
                                }
                            }
                            KeyCode::Backspace => {
                                if let Some(creation_state) = &mut self.creation_state {
                                    match creation_state.current_field {
                                        CreationField::Name => { creation_state.name.pop(); }
                                        CreationField::ProxyType => { creation_state.proxy_type.pop(); }
                                        CreationField::ProxyUrl => { creation_state.proxy_url.pop(); }
                                        CreationField::ProxyPort => { creation_state.proxy_port.pop(); }
                                        CreationField::RedirectPorts => {
                                            creation_state.current_port_input.pop();
                                        }
                                        CreationField::Confirm => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    async fn create_configuration(&mut self) {
        if let Some(creation_state) = &self.creation_state {
            let proxy = Proxy {
                proxy_type: creation_state.proxy_type.clone(),
                url: creation_state.proxy_url.clone(),
                port: creation_state.proxy_port.parse().unwrap_or(0),
            };

            let rules = creation_state.redirect_ports.iter().map(|port| {
                IptablesRule {
                    dport: port.parse().unwrap_or(0),
                    to_port: 14888,
                    action: "REDIRECT".to_string(),
                }
            }).collect();

            let config = Configuration::new(
                creation_state.name.clone(),
                vec![proxy],
                rules,
            ).await;

            self.configurations.push(config);
            self.filter_configurations();
        }
    }

    async fn deactivate_proxy(&mut self) {
        bindings::deactivate_proxy().await;
        self.active_config_index = None;
    }

    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        let title = Paragraph::new("ProxSwap")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("┤ ProxSwap ├")
                .title_alignment(Alignment::Center))
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(35),
                Constraint::Percentage(35),
            ])
            .split(chunks[1]);

        let configs: Vec<ListItem> = self
            .filtered_configs
            .iter()
            .map(|&index| {
                let config = &self.configurations[index];
                let prefix = if Some(index) == self.active_config_index {
                    "● "
                } else {
                    "○ "
                };
                let style = if Some(index) == self.active_config_index {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(format!("{}{}", prefix, config.name)).style(style)
            })
            .collect();

        let configs_list = List::new(configs)
            .block(Block::default()
                .title("Configurations")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(configs_list, main_chunks[0], &mut self.config_list_state.clone());

        if let Some(selected) = self.config_list_state.selected() {
            if let Some(&real_index) = self.filtered_configs.get(selected) {
                let config = &self.configurations[real_index];
                
                let proxies: Vec<ListItem> = config
                    .proxies
                    .iter()
                    .map(|proxy| {
                        ListItem::new(format!(
                            "{} - {}:{}",
                            proxy.proxy_type, proxy.url, proxy.port
                        )).style(Style::default().fg(Color::White))
                    })
                    .collect();

                let proxies_list = List::new(proxies)
                    .block(Block::default()
                        .title("Proxies")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)));

                f.render_widget(proxies_list, main_chunks[1]);

                let rules: Vec<ListItem> = config
                    .rules
                    .iter()
                    .map(|rule| {
                        ListItem::new(format!(
                            "{} → {}",
                            rule.dport, rule.to_port
                        )).style(Style::default().fg(Color::White))
                    })
                    .collect();

                let rules_list = List::new(rules)
                    .block(Block::default()
                        .title("Rules")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)));

                f.render_widget(rules_list, main_chunks[2]);
            }
        }

        let status = match self.input_mode {
            InputMode::Normal => {
                if self.active_config_index.is_some() {
                    "Mode: Normal │ q: quit │ c: create │ x: deactivate proxy │ /: search │ ↑↓: navigate"
                } else {
                    "Mode: Normal │ q: quit │ c: create │ /: search │ ↑↓: navigate"
                }
            }
            InputMode::Editing => "Mode: Editing │ ESC: cancel │ Enter: confirm",
            InputMode::Creating => "Mode: Creating │ ESC: cancel │ ↑/↓: navigate │ Enter: confirm",
        };

        let search_status = if !self.search_query.is_empty() {
            format!("Search: {}", self.search_query)
        } else {
            String::new()
        };

        let status_style = if self.active_config_index.is_some() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let status_bar = Paragraph::new(format!("{} {}", status, search_status))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)))
            .style(status_style);
        f.render_widget(status_bar, chunks[2]);

        if let Some(creation_state) = &self.creation_state {
            let creation_area = centered_rect(60, 20, f.area());
            f.render_widget(Clear, creation_area);
            
            let creation_block = Block::default()
                .title("Create New Configuration")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));

            let mut content = Vec::new();
            
            let style_field = |label: &str, value: &str, is_active: bool| -> Line<'static> {
                let label_span = Span::styled(
                    format!("{}: ", label).to_string(),
                    if is_active {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Gray)
                    }
                );
                let value_span = Span::styled(
                    value.to_string(),
                    if is_active {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                    } else {
                        Style::default().fg(Color::Gray)
                    }
                );
                Line::from(vec![label_span, value_span])
            };

            content.push(style_field("Name", &creation_state.name, 
                matches!(creation_state.current_field, CreationField::Name)));
            content.push(style_field("Proxy Type", &creation_state.proxy_type,
                matches!(creation_state.current_field, CreationField::ProxyType)));
            content.push(style_field("Proxy URL", &creation_state.proxy_url,
                matches!(creation_state.current_field, CreationField::ProxyUrl)));
            content.push(style_field("Proxy Port", &creation_state.proxy_port,
                matches!(creation_state.current_field, CreationField::ProxyPort)));
            
            let ports_str = if creation_state.redirect_ports.is_empty() {
                "None".to_string()
            } else {
                creation_state.redirect_ports.join(", ")
            };
            
            content.push(style_field("Redirect Ports", &ports_str,
                matches!(creation_state.current_field, CreationField::RedirectPorts)));
            
            if matches!(creation_state.current_field, CreationField::RedirectPorts) {
                content.push(style_field("Current Input", &creation_state.current_port_input, true));
            }

            content.push(Line::from(String::from("")));
            content.push(Line::from("─".repeat(40)));
            
            let instructions = vec![
                Line::from(vec![
                    Span::styled("↑/↓".to_string(), Style::default().fg(Color::Yellow)),
                    Span::raw(": Navigate Fields"),
                ]),
                Line::from(vec![
                    Span::styled("Enter".to_string(), Style::default().fg(Color::Yellow)),
                    Span::raw(": Confirm Field/Add Port"),
                ]),
                Line::from(vec![
                    Span::styled("Esc".to_string(), Style::default().fg(Color::Yellow)),
                    Span::raw(": Cancel"),
                ]),
            ];
            content.extend(instructions);

            let text = Text::from(content);
            let paragraph = Paragraph::new(text)
                .block(creation_block)
                .alignment(Alignment::Left);

            f.render_widget(paragraph, creation_area);
        }
    }

    fn next(&mut self) {
        let i = match self.config_list_state.selected() {
            Some(i) => {
                if i >= self.filtered_configs.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.config_list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.config_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_configs.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.config_list_state.select(Some(i));
    }

    fn delete_selected(&mut self) {
        if let Some(selected) = self.config_list_state.selected() {
            if let Some(&real_index) = self.filtered_configs.get(selected) {
                self.configurations.remove(real_index);
                self.filter_configurations();
                if selected >= self.filtered_configs.len() {
                    self.config_list_state.select(Some(self.filtered_configs.len() - 1));
                }
            }
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::ConfigList => Focus::ProxyList,
            Focus::ProxyList => Focus::RulesList,
            Focus::RulesList => Focus::ConfigList,
        };
    }

    fn filter_configurations(&mut self) {
        self.filtered_configs = (0..self.configurations.len())
            .filter(|&i| {
                self.search_query.is_empty()
                    || self.configurations[i]
                        .name
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
            })
            .collect();
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