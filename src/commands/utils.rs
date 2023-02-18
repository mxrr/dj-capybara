pub fn remove_md_characters(s: String) -> String {
  s.replace('_', r"\_")
    .replace('*', r"\*")
    .replace('~', r"\~")
    .replace('`', r"\`")
    .replace('>', r"\>")
    .replace('<', r"\<")
    .replace('[', r"\[")
    .replace(']', r"\]")
}
