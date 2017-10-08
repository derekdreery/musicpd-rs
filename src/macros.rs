
/// Assign a result to a ident, then return ()
macro_rules! grab_val {
    ($id:ident, $iresult:expr) => {
        match $iresult {
            IResult::Done(i, o) => {
                $id = Some(o);
                IResult::Done(i, ())
            }
            IResult::Error(e) => IResult::Error(e),
            IResult::Incomplete(n) => IResult::Incomplete(n),
        }
    }
}

/// Like try!, but for options
macro_rules! try_opt {
    ($opt:expr) => {
        match $opt {
            Some(inner) => inner,
            None => { return None; },
        }
    };
}

