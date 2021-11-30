

pub fn remove_md_characters(s: String) -> String {
  s
    .replace("_", "")
    .replace("*", "")
    .replace("~~", "")
    .replace("`", "")
    .replace(">", "")
    .replace("[", "")
    .replace("]", "")
}
