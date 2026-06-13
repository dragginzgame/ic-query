use std::future::Future;

pub fn block_on_current_thread<F>(future: F) -> Result<F::Output, String>
where
    F: Future,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| err.to_string())?;
    Ok(runtime.block_on(future))
}
