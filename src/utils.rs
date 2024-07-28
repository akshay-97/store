use std::future::Future;

pub async fn time_wrapper<F,U>(func : F,  model_name: &'_ str, op : &'_ str) -> U
where
    F: Future<Output = U>
{
    let start = tokio::time::Instant::now();
    let result = func.await;
    let time_spent = start.elapsed();
    println!("\"{}\" {} {} " , model_name, op,time_spent.as_micros());
    result
}