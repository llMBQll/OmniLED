pub enum Error<T> {
    Recoverable(T),
    Fatal(T),
}
