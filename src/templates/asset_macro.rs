
macro_rules! asset {
  ($x: expr) => {
    match $x {
      "/static/site.css" => "/static/site.536aaadd07.css",
"/static/js/js-enhance.umd.js" => "/static/js/js-enhance.umd.30f9d73f95.js",
      _ => $x,
    }
  }
}
