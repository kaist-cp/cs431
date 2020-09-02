use crossbeam_channel::{bounded, unbounded};
use cs492_concur_homework::hello_server::{
    CancellableTcpListener, Handler, Statistics, ThreadPool,
};
use std::io;
use std::sync::Arc;

const ADDR: &str = "localhost:7878";

fn main() -> io::Result<()> {
    println!("Browse [http://{}]\n", ADDR);

    // The thread pool.
    //
    // In the thread pool, we'll execute:
    //
    // - A listener: it accepts incoming connections, and creates a new worker for each connection.
    //
    // - Workers (once for each incoming connection): a worker handles an incoming connection and
    //   sends a corresponding report to the reporter.
    //
    // - A reporter: it aggregates the reports the reports from the workers and processes the
    //   statistics.  When it ends, it sends the statistics to the main thread.
    let pool = Arc::new(ThreadPool::new(7));

    // The (MPSC) channel of reports between workers and the reporter.
    let (report_sender, report_receiver) = unbounded();

    // The (SPSC one-shot) channel of stats between the reporter and the main thread.
    let (stat_sender, stat_receiver) = bounded(0);

    // Listens to the address.
    let listener = Arc::new(CancellableTcpListener::bind(ADDR)?);

    // Installs a Ctrl-C handler.
    let ctrlc_listner_handle = listener.clone();
    ctrlc::set_handler(move || {
        ctrlc_listner_handle.cancel().unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    // Executes the listener.
    let listener_pool = pool.clone();
    pool.execute(move || {
        // Creates the request handler.
        let handler = Handler::default();

        // For each incoming connection...
        for (id, stream) in listener.incoming().enumerate() {
            // create a new worker thread.
            let report_sender = report_sender.clone();
            let handler = handler.clone();
            listener_pool.execute(move || {
                let report = handler.handle_conn(id, stream.unwrap());
                report_sender.send(report).unwrap();
            });
        }
    });

    // Executes the reporter.
    pool.execute(move || {
        let mut stats = Statistics::default();
        for report in report_receiver {
            println!("[report] {:?}", report);
            stats.add_report(report);
        }

        println!("[sending stat]");
        stat_sender.send(stats).unwrap();
        println!("[sent stat]");
    });

    // Blocks until the reporter sends the statistics.
    let stat = stat_receiver.recv().unwrap();
    println!("[stat] {:?}", stat);

    Ok(())
    // When the pool is dropped, all worker threads are joined.
}
