use actix_web::{web, HttpRequest, HttpResponse, Responder, Result};
use actix_files::NamedFile;
use askama::Template;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::sync::Mutex;

struct State {
  db: &'static str,
  pages: Mutex<BTreeMap<String,Page>>,
}

struct Page {
  link: String,
  title: String,
  time: i64,
  lmod: i64,
  tags: Vec<String>,
  scripts: Vec<String>,
  text: String,
}

#[derive(Template)]
#[template(
  ext = "txt",
  source = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link icon="/pub/favicon.ico">
  <link rel="stylesheet" href="/pub/css/style.css">
  <link rel="preconnect" href="https://fonts.gstatic.com">
  <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Roboto+Mono:ital,wght@0,400;0,700;1,400;1,700&display=swap">
{{analytics}}{{scripts}}</head>
<body>
  <div class="nav">
    {{nav_links}}
  </div>
{{text}}{{foot}}</body>
"#
)]
struct Html {
  analytics: String,
  scripts: String,
  nav_links: String,
  text: String,
  foot: String,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
  use actix_web::{guard, App, HttpServer};

  let db = "db/";
  let state = web::Data::new(State {
    db: db,
    pages: Mutex::new(upstate_all(db)),
  });

  HttpServer::new(move || {
    App::new()
      .app_data(state.clone())
      .service(actix_files::Files::new("/pub", "pub").show_files_listing())
      .route("/favicon.ico", web::get().to(route_fav))
      .route("/", web::get().to(route_root))
      .route("/pages", web::get().to(route_pages))
      .route("/pages/", web::get().to(route_pages))
      .default_service(web::resource("").route(web::get().to(flip))
                       .route(web::route()
                              .guard(guard::Not(guard::Get()))
                              .to(HttpResponse::MethodNotAllowed)))
  })
    .bind("localhost:8000")?
    .run()
    .await
}

async fn route_fav(_: HttpRequest) -> Result<NamedFile> {
  Ok(NamedFile::open("pub/favicon.ico")?)
}

async fn route_root(
  _: web::Data<State>,
  req: HttpRequest
) -> impl Responder {
  use rand::prelude::*;

  let mut rng = rand::thread_rng();
  let n: usize = (6174 / 42 / 3 as usize).pow(2);
  let s: String = std::iter::repeat(())
    .map(|()| rng.sample(rand::distributions::Alphanumeric))
    .map(char::from)
    .take(n)
    .collect();
  let text = String::from(&[r#"  <div class="page welcome">
    <div class="page-head">
      <h1 class="title">welcome</h1>
    </div>
    <div class="page-body">
      <p>"#, s.as_str(), r#"</p>
    </div>
  </div>
"#].join(""));
  let html = html(req.connection_info().host(), "/", "/", text, &vec![]);
  HttpResponse::Ok().body(format!("{}", html.render().unwrap()))
}

async fn route_pages(
  state: web::Data<State>,
  req: HttpRequest
) -> impl Responder {
  let query = req.query_string();
  let re = Regex::new(r"&?tag=([0-9A-za-z]+)").unwrap();
  let input_tag = match re.captures(query) {
    Some(x) => String::from(x.get(1).unwrap().as_str()),
    None => String::new(),
  };
  let all_pages = state.pages.lock().unwrap();
  let pages = all_pages.iter().filter(
    |(_, page)| if input_tag.is_empty() { true } else { page.tags.contains(&input_tag) })
    .collect::<Vec<(&String,&Page)>>();
  let count = pages.len();
  let mut tags = std::collections::BTreeSet::new();
  for (_, page) in pages.iter() {
    for tag in &page.tags {
      tags.insert(tag);
    }
  }
  let text = if 0 == count {
    String::from(r#"  <div class="page">
    <div class="page-head">
      <h1 class="title">pages</h1>
    </div>
    <div class="page-body">
      <p>no pages</p>
    </div>
  </div>
"#)
  } else {
    let mut tags_items = String::new();
    for tag in tags {
      tags_items.push_str(
        format!(r#"        <li><a href="/pages?tag={1}"><div class="tag{0}">{1}</div></a></li>
"#, if input_tag.eq(tag) { " active" } else { "" },
      tag).as_str())
    }
    let tags_count = format!(r#"        <div class="page-count">{} page{}</div>
"#, count, if 1 == count { "" } else { "s" });
    let tags_html = format!(r#"      <ul class="tags-list">{}{}{}      </ul>
"#, if input_tag.is_empty() {
  String::from("tags:\n")
} else {
  String::from(r#"
        <a class="clear" href="/pages"><div class="tag">all tags</div></a>:
"#) }, tags_items, tags_count);
    let mut pages_html = String::new();
    for (name, page) in pages {
      pages_html.push_str(
        format!(r#"        <li><a class="page-link" href="/{}">
          <div class="page-time">
            <div class="time">{}</div><div class="lmod">{}</div>
          </div>
          <div class="name">{}</div>
        </a></li>
"#,
                         page.link,
                         timestamp(page.time),
                         timestamp(page.lmod),
                         name).as_str());
    }
    format!(r#"  <div class="page">
    <div class="page-head">
      <h1 class="title">pages</h1>
    </div>
    <div class="page-body">
{}      <ul class="pages-list">
{}      </ul>
    </div>
  </div>
"#, tags_html, pages_html)
  };
  let html = html(req.connection_info().host(), "pages", "pages", text, &vec![]);
  HttpResponse::Ok().body(format!("{}", html.render().unwrap()))
}

async fn flip(
  state: web::Data<State>,
  req: HttpRequest
) -> impl Responder {
  let db = state.db;
  let mut pages = state.pages.lock().unwrap();
  let conn = req.connection_info();
  let host = conn.host();
  let path = req.match_info().path();
  let mut name = String::from(ammonia::clean(&path[1..]));
  let mut text = name.clone();
  let mut bad = false;
  let mut msg = "";
  let msg_bad = "bad page request";
  let msg_miss = "page not found";

  if 128 < path.len() {
    bad = true;
    name.truncate(127);
    msg = msg_bad;
    text = [String::from(name.clone()), String::from("\u{2026}")].join("");
  }
  if Some(0) == path.find("/page/") {
    name = path.replacen("/page/", "", 1);
  }
  if !Regex::new(r"^[0-9A-Za-z_-]+$").unwrap().is_match(name.as_str()) {
    bad = true;
    msg = msg_bad;
    text = name.clone();
  }
  if !bad && !std::path::Path::new(&[db, name.as_str(), ".md"].join("")).exists() {
    bad = true;
    msg = msg_miss;
    text = name.clone();
  }
  if bad {
    let html = html(host, "", "", format!(r#"  <div class="page">
    <div class="page-head">
      <h1 class="title">oops</h1>
    </div>
    <div class="page-body">
      <p>{}: <b>{}</b></p>
    </div>
  </div>
"#, msg, text), &vec![]);
    return HttpResponse::NotFound().body(format!("{}", html.render().unwrap()));
  }

  upstate(text.clone(), name.clone(), db, &mut pages);
  let html = html(host, text.clone().as_str(), name.as_str(),
                  page_html(text, pages.get_mut(&name).unwrap()),
                  &pages.get(&name).unwrap().scripts);
  HttpResponse::Ok().body(format!("{}", html.render().unwrap()))
}

fn upstate_all(dir: &str) -> BTreeMap<String,Page> {
  let mut pages = BTreeMap::new();
  for f in fs::read_dir(dir).unwrap() {
    let path = f.unwrap().path();
    let mut name = String::from(path.to_str().unwrap());
    if !name.ends_with(".md") {
      continue;
    }
    // TODO: check regex error
    name = String::from(Regex::new(&["^", dir, r"([^.]+)\.md$"].join(""))
      .unwrap().captures(name.as_str()).unwrap().get(1).unwrap().as_str());
    let (title, time, lmod, tags, scripts, text) = parse(name.clone(), path);
    let link = if name.eq("about") || name.eq("dev") {
      name.clone()
    } else {
      format!("page/{}", name)
    };
    pages.insert(name, Page {
      link: link,
      title: title,
      time: time,
      lmod: lmod,
      tags: tags,
      scripts: scripts,
      text: text,
    });
  }
  pages
}

fn upstate(
  link: String,
  name: String,
  db: &str,
  pages: &mut BTreeMap<String,Page>
) {
  let path = &[db, name.as_str(), ".md"].join("");
  if !std::path::Path::new(path).exists() {
    return;
  }
  let meta = fs::metadata(path).unwrap();
  let lmod = meta.modified().unwrap()
    .duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap()
    .as_secs() as i64;
  let mut page = pages.get_mut(&name).unwrap();
  if lmod <= page.lmod {
    return;
  }
  let (title, time, lmod, tags, scripts, text) =
    parse(name, std::path::Path::new(path).to_path_buf());
  page.link = link;
  page.title = title;
  page.time = time;
  page.lmod = lmod;
  page.tags = tags;
  page.scripts = scripts;
  page.text = text;
}

fn parse(
  name: String,
  path: std::path::PathBuf
) -> (String,i64,i64,Vec<String>,Vec<String>,String) {
  use std::io::BufRead;

  let meta = fs::metadata(&path).unwrap();
  let mut title = String::from(Regex::new(r"-+").unwrap()
    .replace(name.as_str(), regex::NoExpand(" ")));
  let mut time = 0;
  let lmod = meta.modified().unwrap()
    .duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap()
    .as_secs() as i64;
  let mut tags: Vec<String> = vec![];
  let mut scripts: Vec<String> = vec![];
  let mut text = String::new();
  let file = io::BufReader::new(fs::File::open(path.as_path()).unwrap());
  let mut done_meta = false;
  for l in file.lines() {
    let line = l.unwrap();
    if line.eq(":::") {
      done_meta = true;
      continue;
    }
    if !done_meta {
      if line.is_empty() {
        continue;
      }
      let re = Regex::new(r"^\s*([^\s:]+):\s+(.*)$").unwrap();
      match re.captures(line.as_str()) {
        Some(x) => {
          match x.get(1).unwrap().as_str() {
            "title" => title = String::from(x.get(2).unwrap().as_str()),
            "time" => time = x.get(2).unwrap().as_str().parse::<i64>().unwrap(),
            "tag" => tags = x.get(2).unwrap().as_str()
              .split(", ").collect::<Vec<_>>().iter()
              .map(|s| String::from(*s)).collect(),
            "script" => scripts = x.get(2).unwrap().as_str()
              .split(", ").collect::<Vec<_>>().iter()
              .map(|s| String::from(*s)).collect(),
            _ => ()
          }
        }
        None => ()
      }
      continue;
    }
    if done_meta {
      text.push_str(line.as_str());
      text.push_str("\n");
    }
  }
  (title, time, lmod, tags, scripts, text)
}

fn html(
  dirty_host: &str,
  dirty_link: &str,
  dirty_name: &str,
  text: String,
  scripts: &Vec<String>,
) -> Html {
  use ammonia::clean;

  let host = clean(dirty_host);
  let link = clean(dirty_link);
  let name = clean(dirty_name);

  let analytics = if host.eq("b.agaric.net") {
    r#"  <script>(function(i,s,o,g,r,a,m){i['GoogleAnalyticsObject']=r;i[r]=i[r]||function(){(i[r].q=i[r].q||[]).push(arguments)},i[r].l=1*new Date();a=s.createElement(o),m=s.getElementsByTagName(o)[0];a.async=1;a.src=g;m.parentNode.insertBefore(a,m)})(window,document,'script','https://www.google-analytics.com/analytics.js','ga');ga('create', 'UA-104505539-1', 'auto');ga('send', 'pageview');</script>
"#
  } else { "" };

  let scripts_line = scripts.iter().map(
    |s| format!(r#"  <script src="/pub/js/{}"></script>
"#, s)).collect();

  let mut nav_links = format!(r#"<div class="root">
      <a {0}href="/"><img class="cup" src="/pub/img/agaric-24.png">b</a>
    </div>
    <div class="links">"#, if link.eq("/") { r#"class="here" "# } else { "" });
  for tab in &["about", "dev", "pages"] {
    nav_links = format!("{0}\n      <a {1}href=\"/{2}\">{2}</a>", nav_links,
                        if !link.is_empty() && link.eq(tab) {
                          r#"class="here" "# } else { "" }, tab);
  }
  nav_links = format!("{}\n    </div>", nav_links);

  let foot = if link.eq("/") {
    String::new()
  } else if link.is_empty() {
    String::from(r#"  <div class="foot">
    <span class="error">404</span>
  </div>
"#)
  } else {
    format!(r#"  <div class="foot">
    <a class="froot" href="/">{0}</a><a href="/{1}">/{1}</a>
  </div>
"#, host, link)
  };

  Html {
    analytics: String::from(analytics),
    scripts: scripts_line,
    nav_links: nav_links,
    text: text,
    foot: foot,
  }
}

fn page_html(name: String, page: &mut Page) -> String {
  let tags = if page.tags.len() == 0 { String::new() } else {
    format!(r#"<div class="tags">
{}      </div>
      "#, page.tags.iter().map(
  |t| format!(r#"        <a href="/pages?tag={0}"><div class="tag">{0}</div></a>
"#,
      t)).collect::<String>())
  };
  format!(r#"  <div class="page _{}">
    <div class="page-head">
      <h1 class="title">{}</h1>
      {}<div class="time">{}</div>
      <div class="lmod">{}</div>
    </div>
    <div class="page-body">
{}    </div>
  </div>
"#,
          name,
          page.title,
          tags,
          timestamp(page.time),
          timestamp(page.lmod),
          md_to_html(&page.text))
}

fn md_to_html(md: &String) -> String {
  let opt = pulldown_cmark::Options::all();
  let parser = pulldown_cmark::Parser::new_ext(md.as_str(), opt);
  let mut html = String::new();
  pulldown_cmark::html::push_html(&mut html, parser);
  html
}

fn timestamp(secs: i64) -> String {
  let then = chrono::NaiveDateTime::from_timestamp(secs, 0);
  format!("{}", then.format("%Y-%m-%d"))
}


