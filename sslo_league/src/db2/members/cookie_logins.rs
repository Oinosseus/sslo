mod row;
mod item;

/// This is the central defined name of the table in this module,
/// used to allow copy&paste of the code for other tables.
macro_rules! tablename {
    () => { "cookie_logins" };
}

pub(self) use tablename;
