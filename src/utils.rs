use std::future::Future;

pub async fn time_wrapper<F,U>(func : F,  model_name: &'_ str, op : &'_ str) -> U
where
    F: Future<Output = U>
{
    let start = tokio::time::Instant::now();
    let result = func.await;
    let time_spent = start.elapsed();
    println!("\"{}\" {} {} " , model_name, op,time_spent.as_micros());
    metrics::histogram!("latency_tracker", &[("model", model_name.to_string()), ("operation", op.to_string())]).record(time_spent.as_secs_f64() * (1000 as f64));
    result
}