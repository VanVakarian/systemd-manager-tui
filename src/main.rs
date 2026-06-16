mod domain;
mod infrastructure;
mod terminal;
mod usecases;
use infrastructure::systemd_service_adapter::{ConnectionType, SystemdServiceAdapter};
use infrastructure::notifier::start_notifier;
use terminal::app::App;
use usecases::services_manager::ServicesManager;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use std::sync::mpsc;

use terminal::app::AppEvent;

use terminal::components::details::ServiceDetails;
use terminal::components::filter::Filter;
use terminal::components::list::TableServices;
use terminal::components::log::ServiceLog;

fn parse_pinned_filters() -> Vec<String> {
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--pinned-filter=") {
            return value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_lowercase())
                .collect();
        }

        if arg == "--pinned-filter" {
            return args
                .next()
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.to_lowercase())
                .collect();
        }
    }

    Vec::new()
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let pinned_filters = parse_pinned_filters();
    
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>();

    start_notifier();
    let systemd_adapter = SystemdServiceAdapter::new(ConnectionType::System)?;
    let usecase = Rc::new(RefCell::new(ServicesManager::new(Box::new(
        systemd_adapter
    ))));
    let table_services = TableServices::new(event_tx.clone(), usecase.clone(), pinned_filters);
    let filter = Filter::new(event_tx.clone());
    let service_log = ServiceLog::new(event_tx.clone(), usecase.clone());
    let details = ServiceDetails::new(event_tx.clone(), usecase.clone());

    let mut app = App::new(
        event_tx,
        event_rx,
        table_services,
        filter,
        service_log,
        details,
        usecase,
    );
    app.init();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
