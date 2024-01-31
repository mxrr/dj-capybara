pub fn remove_md_characters<S>(s: S) -> String
where
  S: ToString,
{
  s.to_string()
    .replace('_', r"\_")
    .replace('*', r"\*")
    .replace('~', r"\~")
    .replace('`', r"\`")
    .replace('>', r"\>")
    .replace('<', r"\<")
    .replace('[', r"\[")
    .replace(']', r"\]")
}
