use actix_files::NamedFile;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use ammonia::clean;
use regex::Regex;
use std::collections::BTreeMap;
use std::{fmt, fs, io};
use std::path::{Path, PathBuf};
use std::result::Result;
use std::sync::Mutex;


static SITE:  &'static str = "b.agaric.net";
static PORT:  &'static str = "8490";
static PUB:   &'static str = "pub";
static SCOPE: &'static str = "page";
static DB:    &'static str = "db";


#[actix_web::main]
async fn main() ->
  io::Result<()>
{
  use actix_web::{App, HttpServer};

  let state = web::Data::new(State {
    site: SITE.to_string(),
    dir: DB.to_string(),
    scope: SCOPE.to_string(),
    pages: Mutex::new(BTreeMap::new()),
  });
  for dir in [&state.dir, PUB].iter() {
    if !Path::new(dir).is_dir() {
      return Err(io::Error::new(io::ErrorKind::NotFound,
                                ["not a directory: ", dir].join("")));
    }
  }
  state.up_state("", "").unwrap_or_else(|e| println!("Err: main/{}", e));
  let pages_route = ["/", &state.scope, "s{_:/?}"].join("");
  let page_route = ["/", &state.scope, "/{page}"].join("");

  HttpServer::new(move || {
    App::new()
      .app_data(state.clone())
      .service(actix_files::Files::new(&["/", PUB].join(""),
                                       &PUB).show_files_listing())
      .route("/favicon.ico",      web::get().to(route_fav))
      .route("/",                 web::get().to(route_root))
      .route(&pages_route,        web::get().to(route_pages))
      .route("/{page:about}",     web::get().to(flip))
      .route("/{page:dev}{_:/?}", web::get().to(flip))
      .route(&page_route,         web::get().to(flip))
      .default_service(web::get().to(route_miss))
  }).bind(["localhost:", PORT].join(""))?.run().await
}


/*** routing *****************************************************************/

async fn route_miss(
  state: web::Data<State>,
  req: HttpRequest,
) -> impl Responder
{
  let name = &bound(req.match_info().path())[1..];
  respond_error(404, &state.site, name, "page not found", name)
}


async fn route_fav(_req: HttpRequest) ->
  actix_web::Result<NamedFile>
{
  Ok(NamedFile::open(&[PUB, "/favicon.ico"].join(""))?)
}


async fn route_root(
  state: web::Data<State>,
  _req: HttpRequest,
) -> impl Responder
{
  HttpResponse::Ok().body(
    html(/* error   */ "",
         /* site    */ &state.site,
         /* link    */ "/",
         /* query   */ "",
         /* scripts */ &vec![],
         /* title   */ "",
         /* html    */ &html_pagelet(/* class */ "-welcome",
                                     /* title */ "welcome",
                                     /* text  */ &root_motd(),
                                     /* more  */ "")))
}


async fn route_pages(
  state: web::Data<State>,
  req: HttpRequest,
) -> impl Responder
{
  let site = &state.site;
  state.up_state("", "")
    .unwrap_or_else(|e| println!("Err: route_pages/{}", e));
  let mut pages = match state.pages.lock() {
    Ok(o) => o,
    Err(_) => {
      println!("Err: route_pages: pages lock");
      return respond_error(500, site, "pages", "", "pages");
    },
  };

  let pages_total = pages.len();
  let (err, query, tags_list, pages_list) =
    &pages_lists(&clean(&bound(req.query_string())), &mut pages);
  if *err || (0 == pages_list.len() && 0 < pages_total) {
    return respond_error(404, site, "pages", "no pages tagged", query);
  }

  HttpResponse::Ok().body(
    html(/* error   */ "",
         /* site    */ site,
         /* link    */ "pages",
         /* query   */ &query,
         /* scripts */ &vec![],
         /* title   */ "pages",
         /* html    */ &html_pages(/* query */ &query,
                                   /* total */ pages_total,
                                   /* tags  */ tags_list,
                                   /* pages */ pages_list)))
}


async fn flip(
  state: web::Data<State>,
  req: HttpRequest,
) -> impl Responder
{
  let site = &state.site;
  let dir = &state.dir;
  let scope = &state.scope;
  let n: String = req.match_info().get(scope).unwrap().parse().unwrap();
  let name = &bound(&n);

  if !Regex::new(r"^[0-9A-Za-z_-]+$").unwrap().is_match(name) ||
     !page_exists(dir, name)
  {
    return respond_error(404, site, name, "page not found", name);
  }

  state.up_state(&[scope, "/", name].join(""), name)
    .unwrap_or_else(|e| println!("Err: flip/{}", e));

  let pages = match state.pages.lock() {
    Ok(o) => o,
    Err(_) => {
      println!("Err: flip: pages lock");
      return respond_error(500, site, name, "", name);
    },
  };

  let page = match pages.get(name) {
    Some(o) => o,
    None => {
      println!("Err: flip: get page");
      return respond_error(500, site, name, "", name);
    },
  };

  HttpResponse::Ok().body(&page.html)
}


/*** error *******************************************************************/

struct Error(String);

impl fmt::Display for Error
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    f.write_str(&self.to_string())
  }
}


fn err(fun: &str, what: &str) ->
  Error
{
  Error(format!("{}: {}", fun, what))
}


/*** state *******************************************************************/

struct Page
{
  name: String,
  link: String,
  //title: String,
  time: i64,
  lmod: i64,
  tags: Vec<String>,
  //scripts: Vec<String>,
  html: String,
}


struct State
{
  site: String,
  dir: String,
  scope: String,
  pages: Mutex<BTreeMap<String,Page>>,
}

impl State
{
  pub fn up_state(&self, link: &str, name: &str) ->
    Result<(),Error>
  {
    if name.is_empty() {
      self.up_pages()
    } else {
      self.up_page(link, name)
    }
  }


  pub fn up_pages(&self) ->
    Result<(),Error>
  {
    let mut pages = self.pages.lock()
      .or_else(|_| Err(err("up_pages", "pages lock")))?;
    for (page_link, page_name, path) in &self.up_db(&mut pages)? {
      let page = self.page(&page_name, &page_link, path.to_path_buf())?;
      pages.insert(page_name.to_string(), page);
    }
    Ok(())
  }


  pub fn up_page(&self, link: &str, name: &str) ->
    Result<(),Error>
  {
    let mut pages = self.pages.lock()
      .or_else(|_| Err(err("up_page", "pages lock")))?;
    let page = pages.get(name)
      .ok_or_else(|| err("up_page", "get page"))?;
    if self.page_is_old(&page)? {
      let path = Path::new(&page_path(&self.dir, &name)).to_path_buf();
      let page = self.page(name, link, path)?;
      pages.insert(name.to_string(), page);
    }
    Ok(())
  }


  pub fn up_db(
    &self,
    pages: &mut BTreeMap<String,Page>
  ) -> Result<Vec<(String,String,PathBuf)>,Error>
  {
    // remove nonexistent pages
    // TODO: use BTreeMap.drain_filter() rather than reparse everything
    for (name, _) in pages.iter() {
      if !page_exists(&self.dir, &name) {
        pages.clear();
        break;
      }
    }

    // detect files to parse anew
    let mut db = vec![];
    let files = fs::read_dir(&self.dir)
      .or_else(|_| Err(err("up_db", "read_dir")))?;
    for f in files {
      // good file extension
      let file = f.or_else(|_| Err(err("up_db", "file")))?;
      let path = file.path();
      let mut name = path.to_str().unwrap().to_string();
      if !name.ends_with(".md") {
        continue;
      }

      // sane charset
      name = match Regex::new(&["^", &self.dir, r"/([^.]+)\.md$"].join(""))
        .unwrap().captures(&name)
      {
        Some(x) => match x.get(1) {
          Some(x) => x.as_str().to_string(),
          None => continue,
        },
        None => continue,
      };
      let link = if name.eq("about") || name.eq("dev") {
        name.clone()
      } else {
        ["page/", &name].join("")
      };

      // page older in state than in db
      if pages.contains_key(name.as_str()) {
        let meta = file.metadata()
          .or_else(|_| Err(err("up_db", "file metadata")))?;
        let page = pages.get(&name)
          .ok_or_else(|| err("up_db", "get page"))?;
        if page_lmod(meta) <= page.lmod {
          continue;
        }
      }

      db.push((link, name, path));
    }

    Ok(db)
  }


  pub fn page(
    &self,
    name: &str,
    link: &str,
    path: PathBuf,
  ) -> Result<Page,Error>
  {
    let (title, time, lmod, tags, scripts, md) = parse(name, path)?;
    let html_page = html_page(/* scope */ &self.scope,
                              /* name  */ name,
                              /* title */ &title,
                              /* time  */ time,
                              /* lmod  */ lmod,
                              /* tags  */ &tags,
                              /* belly */ &md);

    Ok(Page {
      name: name.to_string(),
      link: link.to_string(),
      time: time,
      lmod: lmod,
      tags: tags,
      html: html(/* error   */ "",
                 /* site    */ &self.site,
                 /* link    */ &link,
                 /* query   */ "",
                 /* scripts */ &scripts,
                 /* title   */ &title,
                 /* html    */ &html_page),
    })
  }


  pub fn page_is_old(&self, page: &Page) ->
    Result<bool,Error>
  {
    let path = page_path(&self.dir, &page.name);
    if !page_exists(&self.dir, &page.name) {
      return Ok(false);
    }

    let meta = fs::metadata(path)
      .or_else(|_| Err(err("page_is_old", "file metadata")))?;
    Ok(page_lmod(meta) > page.lmod)
  }
}


/*** page helpers ************************************************************/

fn respond_error(
  code: i32,
  site: &str,
  link: &str,
  mut text: &str,
  more: &str,
) -> HttpResponse
{
  let (error, mut response) = match code {
    500 => {
      text = "trouble processing";
      ("500", HttpResponse::InternalServerError())
    },
    _ => {
      ("404", HttpResponse::NotFound())
    },
  };

  response.body(
    html(/* error   */ error,
         /* site    */ site,
         /* link    */ link,
         /* query   */ "",
         /* scripts */ &vec![],
         /* title   */ "",
         /* html    */ &html_pagelet(/* class */ error,
                                     /* title */ "oops",
                                     /* text  */ text,
                                     /* more  */ more)))
}


fn root_motd() ->
  String
{
  use rand::prelude::*;

  let mut rng = rand::thread_rng();
  let kaprekar = 6174;
  let answer = 42;
  let trinity = 3;
  let plutonium = kaprekar / answer / trinity;
  let n = (plutonium * plutonium - 1) / 2;

  std::iter::repeat(())
    .map(|()| rng.sample(rand::distributions::Alphanumeric))
    .map(char::from)
    .take(n)
    .collect()
}


fn pages_lists<'a>(
  input_query: &str,
  pages: &'a mut BTreeMap<String,Page>,
) -> (bool,String,BTreeMap<&'a str,i16>,Vec<(&'a String,&'a Page)>)
{
  use std::collections::BTreeSet;

  if !input_query.is_empty() &&
     input_query.chars().last().unwrap() == '\u{2026}'
  {
    return (true, input_query.to_string(), BTreeMap::new(), vec![]);
  }
  let query = match Regex::new(r"[?&]?tag=([0-9A-za-z]+)").unwrap()
    .captures(input_query)
  {
    Some(x) => match x.get(1) {
      Some(x) => x.as_str().to_string(),
      None => String::new(),
    },
    None => String::new(),
  };

  let mut tags_take: BTreeSet<&str> = BTreeSet::new();
  let pages_list = pages.iter().filter(
    |(_name, page)| {
      let mut take = true;
      if !query.is_empty() {
        take = page.tags.contains(&query);
      }
      if take {
        for tag in page.tags.iter() {
          tags_take.insert(&tag);
        }
      }
      take
    }).collect::<Vec<(&String,&Page)>>();

  let mut tags_list: BTreeMap<&str,i16> = BTreeMap::new();
  for (_name, page) in pages.iter() {
    for tag in page.tags.iter() {
      if tags_take.contains(tag.as_str()) {
        *tags_list.entry(tag).or_insert(0) += 1;
      }
    }
  }

  (false, query, tags_list, pages_list)
}


/*** utility *****************************************************************/

fn parse(
  name: &str,
  path: PathBuf,
) -> Result<(String,i64,i64,Vec<String>,Vec<String>,String),Error>
{
  use std::io::BufRead;

  let mut title = Regex::new(r"[_-]+").unwrap()
    .replace(name, regex::NoExpand(" ")).to_string();
  let mut time = 0;
  let meta = fs::metadata(&path)
    .or_else(|_| Err(err("parse", "file metadata")))?;
  let sep = Regex::new(r",\s*").unwrap();
  let mut tags = vec![];
  let mut scripts = vec![];
  let mut md = String::with_capacity(512);
  let mut done_meta = false;
  let file = fs::File::open(path.as_path())
    .or_else(|_| Err(err("parse", "file open")))?;
  for l in io::BufReader::new(file).lines() {
    let line = l.unwrap();
    if Regex::new(r"^[:\s]*:::+[:\s]*$").unwrap().is_match(&line) {
      done_meta = true;
      continue;
    }
    if !done_meta {
      if line.is_empty() {
        continue;
      }
      match Regex::new(r"^\s*([^\s:]+):\s+(.*)$").unwrap().captures(&line) {
        Some(caps) => {
          let val = match caps.get(2) {
            Some(x) => x.as_str(),
            None => continue,
          };
          match caps.get(1) {
            Some(x) => match x.as_str() {
              "title" => title = val.to_string(),
              "time" => time = val.parse::<i64>().unwrap(),
              "tag" => tags = sep.split(val).collect::<Vec<_>>().iter()
                .map(|s| (*s).to_string()).collect(),
              "script" => scripts = sep.split(val).collect::<Vec<_>>().iter()
                .map(|s| (*s).to_string()).collect(),
              _ => continue,
            },
            None => continue,
          }
        },
        None => continue,
      }
      continue;
    }
    md.push_str(&line);
    md.push_str("\n");
  }

  Ok((title, time, page_lmod(meta), tags, scripts, md_to_html(&md)))
}


fn md_to_html(md: &str) ->
  String
{
  use pulldown_cmark::{html, Options, Parser};

  let mut opts = Options::empty();
  opts.insert(Options::ENABLE_TABLES);
  opts.insert(Options::ENABLE_FOOTNOTES);
  opts.insert(Options::ENABLE_STRIKETHROUGH);
  opts.insert(Options::ENABLE_TASKLISTS);
  //opts.insert(Options::ENABLE_SMART_PUNCTUATION);
  let parser = Parser::new_ext(md, opts);
  let mut html_md = String::new();
  html::push_html(&mut html_md, parser);

  html_md
}


fn page_exists(dir: &str, name: &str) ->
  bool
{
  Path::new(&page_path(dir, name)).is_file()
}


fn page_path(dir: &str, name: &str) ->
  String
{
  [dir, "/", name, ".md"].join("")
}


fn page_lmod(meta: fs::Metadata) ->
  i64
{
  meta.modified().unwrap()
    .duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap()
    .as_secs() as i64
}


fn timestamp(secs: i64) ->
  String
{
  let then = chrono::NaiveDateTime::from_timestamp(secs, 0);

  then.format("%Y-%m-%d").to_string()
}


fn titlestamp(error: bool, site: &str, link: &str, title: &str, tag: &str) ->
  String
{
  if error || link.eq("/") {
    return site.to_string();
  }

  let mut s = String::new();
  if title.is_empty() {
    s.push_str(link);
  } else {
    s.push_str(title);
  }
  if !tag.is_empty() {
    s.push_str(":");
    s.push_str(tag);
  }
  s.push_str(" - ");
  s.push_str(site);

  s
}


fn bound<S: Into<String>>(s: S) ->
  String
{
  let max = 64;
  let mut b = s.into();
  match b.char_indices().nth(max) {
    Some((i, _)) => {
      b = b[..i - 1].to_string();
      b.push_str("\u{2026}");
      b
    },
    None => b,
  }
}


/*** html generation *********************************************************/

fn html(
  error: &str,           // error code
  site: &str,            // admin-defined site name
  link: &str,            // user-requested path
  query: &str,           // user-requested query
  scripts: &Vec<String>, // optional page scripts
  title: &str,           // page title
  belly: &str,           // the html going into body > .page > .page-body
) -> String
{
  let mut s = String::with_capacity(16384);
  s.push_str(r#"<!doctype html>
<html lang="en">
<head>
"#);
  s.push_str(&html_analytics());
  s.push_str(r#"  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
"#);
  s.push_str("  <title>");
  s.push_str(&titlestamp(!error.is_empty(), site, &link, title, &query));
  s.push_str(r#"</title>
"#);
  s.push_str(&html_social(site, &link, title, &query));
  s.push_str(r#"  <link icon="/"#);
  s.push_str(PUB);
  s.push_str(r#"/favicon.ico">
  <link rel="stylesheet" href="/"#);
  s.push_str(PUB);
  s.push_str(r#"/css/style.css">
  <link rel="preconnect" href="https://fonts.gstatic.com">
  <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Roboto+Mono:ital,wght@0,400;0,700;1,400;1,700&display=swap">
"#);
  s.push_str(&html_scripts(scripts));
  s.push_str(r#"</head>
<body>
"#);
  s.push_str(&html_nav(&link));
  s.push_str(&belly);
  s.push_str(&html_foot(error, &query, &link));
  s.push_str(r#"</body>
"#);

  s
}


fn html_pages(
  query: &str,
  total: usize,
  tags: &BTreeMap<&str,i16>,
  pages: &Vec<(&String,&Page)>,
) -> String
{
  if pages.is_empty() {
    return html_pagelet("", "pages", "no pages to list", "");
  }

  let mut html = String::with_capacity(12288);
  html.push_str(r#"  <div class="page -pages">
    <div class="page-head">
      <h1 class="title">pages</h1>
    </div>
    <div class="page-body">
      <ul class="tags">
"#);
  html.push_str(r#"        <li><a class="tag clear"#);
  if query.is_empty() {
    html.push_str(" active");
  }
  html.push_str(r#"" href="/pages"><span class="tag-name">all</span><span class="tag-count">"#);
  html.push_str(&total.to_string());
  html.push_str("</span></a></li>\n");
  for (tag, count) in tags {
    html.push_str(r#"        <li><a class="tag"#);
    if tag.eq(&query) {
      html.push_str(" active");
    }
    html.push_str(r#"" href="/pages?tag="#);
    html.push_str(tag);
    html.push_str(r#""><span class="tag-name">"#);
    html.push_str(tag);
    html.push_str(r#"</span><span class="tag-count">"#);
    html.push_str(&count.to_string());
    html.push_str(r#"</span></a></li>
"#);
  }
  html.push_str(r#"      </ul>
      <ul class="pages">
"#);
  for (name, page) in pages {
    html.push_str(r#"        <li>
          <div class="page-time">
            <div class="time">"#);
    html.push_str(&timestamp(page.time));
    html.push_str(r#"</div><div class="lmod">"#);
    html.push_str(&timestamp(page.lmod));
    html.push_str(r#"</div>
          </div>
          <a class="name" href="/"#);
    html.push_str(&page.link);
    html.push_str(r#"">"#);
    html.push_str(name);
    html.push_str("</a>\n");
    if 0 < page.tags.len() {
      html.push_str(r#"          <ul class="page-tags">
"#);
    }
    for tag in &page.tags {
      html.push_str(r#"            <li><a class="tag"#);
      if tag.eq(query) {
        html.push_str(" active");
      }
      html.push_str(r#"" href="/pages?tag="#);
      html.push_str(&tag);
      html.push_str(r#"">"#);
      html.push_str(&tag);
      html.push_str("</a></li>\n");
    }
    if 0 < page.tags.len() {
      html.push_str("          </ul>\n");
    }
    html.push_str("        </li>\n");
  }
  html.push_str(r#"      </ul>
    </div>
  </div>
"#);

  html
}


fn html_page(
  scope: &str,
  name: &str,
  title: &str,
  time: i64,
  lmod: i64,
  tags: &Vec<String>,
  md: &str,
) -> String
{
  let mut html = String::with_capacity(1024);
  html.push_str(r#"  <div class="page _"#);
  html.push_str(name);
  html.push_str(r#"">
    <div class="page-head">
      <h1 class="title">"#);
  html.push_str(title);
  html.push_str("</h1>\n");
  if 0 < tags.len() {
    html.push_str(r#"      <ul class="tags">
"#);
    for tag in tags {
      html.push_str(r#"        <li><a class="tag" href="/"#);
      html.push_str(scope);
      html.push_str("s?tag=");
      html.push_str(tag);
      html.push_str(r#"">"#);
      html.push_str(tag);
      html.push_str("</a></li>\n");
    }
    html.push_str("      </ul>\n");
  }
  html.push_str(r#"      <div class="time">"#);
  html.push_str(&timestamp(time));
  html.push_str(r#"</div>
      <div class="lmod">"#);
  html.push_str(&timestamp(lmod));
  html.push_str(r#"</div>
    </div>
    <div class="page-body">
"#);
  html.push_str(md);
  html.push_str(r#"    </div>
  </div>
"#);

  html
}


fn html_pagelet(class: &str, title: &str, text: &str, more: &str) ->
  String
{
  let long = 48;
  let mut html = String::with_capacity(256);
  html.push_str(r#"  <div class="page"#);
  if !class.is_empty() {
    html.push_str(" ");
    if class.chars().next().unwrap().is_numeric() {
      html.push_str("e");
    }
    html.push_str(class);
  }
  html.push_str(r#"">
    <div class="page-head">
      <h1 class="title">"#);
  html.push_str(title);
  html.push_str(r#"</h1>
    </div>
    <div class="page-body">
      <p>"#);
  html.push_str(text);
  if !more.is_empty() {
    html.push_str(": ");
    if long < more.len() {
      html.push_str("<br/>");
    }
    html.push_str("<b>");
    html.push_str(more);
    html.push_str("</b>");
  }
  html.push_str(r#"</p>
    </div>
  </div>
"#);

  html
}


fn html_analytics() ->
  String
{
  //String::new()
  r#"  <script async src="https://www.googletagmanager.com/gtag/js?id=G-PT10SS3WP3"></script><script>window.dataLayer = window.dataLayer || [];function gtag(){dataLayer.push(arguments);}gtag('js', new Date());gtag('config', 'G-PT10SS3WP3');</script>
"#.to_string()
}


fn html_social(site: &str, link: &str, title: &str, tag: &str) ->
  String
{
  let mut html = String::new();
  let image = &["https://", site, "/", PUB, "/img/agaric-512.png"].join("");
  let riddle = "What room has no walls?";
  let full = &[site, "/", link].join("");
  let full_tag = &[full, ":", tag].join("");
  let stamp = &titlestamp(false, site, link, title, tag);
  let short = if title.is_empty() {
    site
  } else {
    stamp
  };
  let req = if title.is_empty() {
    riddle
  } else if tag.is_empty() {
    full
  } else {
    full_tag
  };
  html.push_str(r#"  <meta name="description" content=""#);
  html.push_str(req);
  html.push_str(r#"">
  <meta name="image" content=""#);
  html.push_str(image);
  html.push_str(r#"">
  <meta itemprop="name" content=""#);
  html.push_str(short);
  html.push_str(r#"">
  <meta itemprop="description" content=""#);
  html.push_str(req);
  html.push_str(r#"">
  <meta itemprop="image" content=""#);
  html.push_str(image);
  html.push_str(r#"">
  <meta name="twitter:card" content="summary">
  <meta name="twitter:title" content=""#);
  html.push_str(short);
  html.push_str(r#"">
  <meta name="twitter:description" content=""#);
  html.push_str(req);
  html.push_str(r#"">
  <meta name="og:title" content=""#);
  html.push_str(short);
  html.push_str(r#"">
  <meta name="og:description" content=""#);
  html.push_str(req);
  html.push_str(r#"">
  <meta name="og:image" content=""#);
  html.push_str(image);
  html.push_str(r#"">
  <meta name="og:url" content=""#);
  html.push_str(&["https://", req].join(""));
  html.push_str(r#"">
  <meta name="og:site_name" content=""#);
  html.push_str(short);
  html.push_str(r#"">
  <meta name="og:locale" content="en_US">
  <meta name="og:type" content="website">
"#);

  html
}


fn html_scripts(scripts: &Vec<String>) ->
  String
{
  let mut html = String::new();
  for s in scripts {
    html.push_str(r#"  <script src="/"#);
    html.push_str(PUB);
    html.push_str("/js/");
    html.push_str(s);
    html.push_str(r#""></script>
"#);
  }

  html
}


fn html_nav(link: &str) ->
  String
{
  let mut html = String::with_capacity(512);
  html.push_str(r#"  <div class="nav">
    <div class="root">
      <a "#);
  if link.eq("/") {
    html.push_str(r#"class="here" "#);
  }
  html.push_str(r#"href="/"><img class="cup" src="/"#);
  html.push_str(PUB);
  html.push_str(r#"/img/agaric-24.png">b</a>
    </div>
    <div class="links">
"#);
  for name in ["about", "dev", "pages"].iter() {
    html.push_str("      <a ");
    if !link.is_empty() && name.eq(&link) {
      html.push_str(r#"class="here" "#);
    }
    html.push_str(r#"href="/"#);
    html.push_str(name);
    html.push_str(r#"">"#);
    html.push_str(name);
    html.push_str("</a>\n");
  }
  html.push_str(r#"    </div>
  </div>
"#);

  html
}


fn html_foot(error: &str, query: &str, link: &str) ->
  String
{
  if link.eq("/") {
    return String::new();
  }

  let mut html = String::with_capacity(256);
  html.push_str(r#"  <div class="foot">
"#);
  if !error.is_empty() {
    html.push_str(r#"    <span class="error">"#);
    html.push_str(error);
    html.push_str("</span>\n");
  }
  html.push_str(r#"    <span class="stride"></span>
    <div class="lace"><a class="froot" href="/">root</a></span><a href=""#);
  html.push_str("/");
  html.push_str(link);
  if !query.is_empty() {
    html.push_str("?tag=");
    html.push_str(query);
  };
  html.push_str(r#"">/"#);
  html.push_str(link);
  if !query.is_empty() {
    html.push_str(":");
    html.push_str(query);
  };
  html.push_str(r#"</a></div>
  </div>
"#);

  html
}

