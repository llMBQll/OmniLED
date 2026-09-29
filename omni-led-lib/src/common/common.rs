#[macro_export]
macro_rules! create_table {
    ($lua:ident, $values:tt) => {
        $lua.load(mlua::chunk! { $values })
            .eval::<mlua::Table>()
            .unwrap()
    };
}
