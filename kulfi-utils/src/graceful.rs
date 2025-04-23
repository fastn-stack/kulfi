use eyre::Context;
use tokio::task::JoinHandle;

#[derive(Clone, Default)]
pub struct Graceful {
    cancel: tokio_util::sync::CancellationToken,
    tracker: tokio_util::task::TaskTracker,
}

impl Graceful {
    #[inline]
    #[track_caller]
    pub fn spawn<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tracker.spawn(task)
    }

    pub async fn shutdown(
        &self,
        show_info_tx: tokio::sync::watch::Sender<bool>,
    ) -> eyre::Result<()> {
        loop {
            tokio::signal::ctrl_c()
                .await
                .wrap_err_with(|| "failed to get ctrl-c signal handler")?;

            tracing::info!("Received ctrl-c signal, showing info.");
            tracing::info!("Pending tasks: {}", self.tracker.len());

            show_info_tx
                .send(true)
                .inspect_err(|e| tracing::error!("failed to send show info signal: {e:?}"))?;

            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Received second ctrl-c signal, shutting down.");
                    tracing::debug!("Pending tasks: {}", self.tracker.len());

                    self.cancel.cancel();
                    self.tracker.close();

                    let mut count = 0;
                    loop {
                        tokio::select! {
                            _ = self.tracker.wait() => {
                                tracing::info!("All tasks have exited.");
                                break;
                            }
                            _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
                                count += 1;
                                if count > 10 {
                                    eprintln!("Timeout expired, {} pending tasks. Exiting...", self.tracker.len());
                                    break;
                                }
                                tracing::debug!("Pending tasks: {}", self.tracker.len());
                            }
                        }
                    }
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
                    tracing::info!("Timeout expired. Continuing...");
                    println!("Did not receive ctrl+c within 3 secs. Press ctrl+c in quick succession to exit.");
                }
            }
        }

        Ok(())
    }

    pub fn cancelled(&self) -> tokio_util::sync::WaitForCancellationFuture<'_> {
        self.cancel.cancelled()
    }
}
