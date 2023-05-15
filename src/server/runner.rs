use std::future::Future;

struct Runner<T>
where T: Future
{
    future: T,
}