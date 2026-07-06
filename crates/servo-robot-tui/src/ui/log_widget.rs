//! 日志显示组件（基于 tui-logger）

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use tui_logger::TuiLoggerWidget;
use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let title = if app.log_focus {
        "📝Board Log [FocusMode] [(Esc)Exist] [(↑↓) Page] [(←→)Filter]"
    } else {
        "📝Board Log [(F)ocus]"
    };

    let border_style = if app.log_focus {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Magenta)
    };

    let mut widget = TuiLoggerWidget::default()
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style_error(Style::default().fg(Color::Red))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_info(Style::default().fg(Color::Green))
        .style_debug(Style::default().fg(Color::DarkGray))
        .style_trace(Style::default().fg(Color::White));

    if app.log_focus {
        widget = widget.state(&app.log_state);
    }

    f.render_widget(widget, area);
}
